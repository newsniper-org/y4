// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `ConfigSpace` — abstraction over `PCIe` configuration-space access.
//!
//! Real production: x86 CAM (0xCF8 / 0xCFC) or ECAM (memory-mapped
//! config region).  Mock: a sparse map of (bus, dev, fn) → 256-byte
//! header used by the unit tests.  The trait surface is the minimum
//! every implementor needs: read 4 bytes at an arbitrary header offset.

/// `PCIe` (bus, device, function, offset) identifies one 4-byte word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigAddr {
    /// Bus number (0..=255).
    pub bus: u8,
    /// Device number (0..=31).
    pub dev: u8,
    /// Function number (0..=7).
    pub func: u8,
    /// DWORD offset within the device's 256-byte (or 4096-byte) header.
    pub offset: u8,
}

impl ConfigAddr {
    /// Pack `(bus, dev, fn)` into a single `u32` Y4 uses for `BusAddr`
    /// values (per `proofs/verus/src/capsules/state.rs`).
    #[must_use]
    pub fn pack_bdf(self) -> u32 {
        (u32::from(self.bus) << 16) | (u32::from(self.dev) << 11) | (u32::from(self.func) << 8)
    }
}

/// `PCIe` config-space read surface.
pub trait ConfigSpace {
    /// Read a 32-bit dword from `addr`.  Returns `0xFFFF_FFFF` for
    /// unimplemented (bus, dev, fn) — that's the spec-mandated "device
    /// absent" pattern.
    fn read_u32(&self, addr: ConfigAddr) -> u32;

    /// Convenience reader for a 16-bit half (caller picks even
    /// `byte_offset`).
    fn read_u16(&self, bus: u8, dev: u8, func: u8, byte_offset: u8) -> u16 {
        let aligned = ConfigAddr {
            bus,
            dev,
            func,
            offset: byte_offset / 4,
        };
        let word = self.read_u32(aligned);
        let shift = (byte_offset % 4) * 8;
        ((word >> shift) & 0xFFFF) as u16
    }

    /// Convenience reader for a single byte.
    fn read_u8(&self, bus: u8, dev: u8, func: u8, byte_offset: u8) -> u8 {
        let aligned = ConfigAddr {
            bus,
            dev,
            func,
            offset: byte_offset / 4,
        };
        let word = self.read_u32(aligned);
        let shift = (byte_offset % 4) * 8;
        ((word >> shift) & 0xFF) as u8
    }
}

/// One device the test harness wants to expose to a `MockConfigSpace`.
#[derive(Debug, Clone, Copy)]
pub struct MockDevice {
    /// Bus / device / function this device sits at.
    pub bus: u8,
    /// Device number.
    pub dev: u8,
    /// Function number.
    pub func: u8,
    /// 16-bit vendor id (offset 0x00).
    pub vendor_id: u16,
    /// 16-bit device id (offset 0x02).
    pub device_id: u16,
    /// 24-bit class code (offset 0x09–0x0B), packed into the low bits.
    pub class_code: u32,
    /// PCI header type at offset 0x0E.  `0x00` = endpoint, `0x01` =
    /// PCI-PCI bridge.  Bit 7 set = multi-function device.
    pub header_type: u8,
    /// Secondary bus number (offset 0x19) when this device is a bridge.
    /// Ignored when `header_type & 0x7F != 0x01`.
    pub secondary_bus: u8,
}

/// Mock config-space backed by a flat list of devices.  Used by both
/// the `pcie::tests` module and any downstream capsule wanting to
/// rehearse against synthetic topologies.
#[derive(Debug, Default)]
pub struct MockConfigSpace {
    devices: heapless::Vec<MockDevice, 64>,
}

impl MockConfigSpace {
    /// Empty config space — every read returns `0xFFFF_FFFF`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `dev` so that subsequent reads against
    /// `(dev.bus, dev.dev, dev.func)` return live header bytes.
    ///
    /// # Errors
    /// Returns the device unchanged if the internal vec is full.
    pub fn add(&mut self, dev: MockDevice) -> Result<(), MockDevice> {
        self.devices.push(dev)
    }

    fn lookup(&self, bus: u8, dev: u8, func: u8) -> Option<&MockDevice> {
        self.devices
            .iter()
            .find(|d| d.bus == bus && d.dev == dev && d.func == func)
    }
}

impl ConfigSpace for MockConfigSpace {
    fn read_u32(&self, addr: ConfigAddr) -> u32 {
        let Some(d) = self.lookup(addr.bus, addr.dev, addr.func) else {
            return 0xFFFF_FFFF;
        };
        match addr.offset {
            // 0x00–0x03: vendor (low) | device (high)
            0 => u32::from(d.vendor_id) | (u32::from(d.device_id) << 16),
            // 0x08–0x0B: revision (skipped, 0) | class_code (high 24)
            2 => d.class_code << 8,
            // 0x0C–0x0F: cache_line | latency | header_type | BIST
            3 => u32::from(d.header_type) << 16,
            // 0x18–0x1B: primary | secondary | subordinate | latency_timer
            // for type-1 (bridge) headers.
            6 => {
                if (d.header_type & 0x7F) == 0x01 {
                    u32::from(d.secondary_bus) << 8
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_device_returns_ffff() {
        let cs = MockConfigSpace::new();
        let v = cs.read_u32(ConfigAddr {
            bus: 0,
            dev: 0,
            func: 0,
            offset: 0,
        });
        assert_eq!(v, 0xFFFF_FFFF);
    }

    #[test]
    fn vendor_device_round_trip() {
        let mut cs = MockConfigSpace::new();
        cs.add(MockDevice {
            bus: 0,
            dev: 1,
            func: 0,
            vendor_id: 0x10DE, // NVIDIA
            device_id: 0x1234,
            class_code: 0x03_0000, // VGA
            header_type: 0x00,
            secondary_bus: 0,
        })
        .unwrap();
        let v = cs.read_u16(0, 1, 0, 0x00);
        let d = cs.read_u16(0, 1, 0, 0x02);
        assert_eq!(v, 0x10DE);
        assert_eq!(d, 0x1234);
    }

    #[test]
    fn pack_bdf_field_layout() {
        // The packed format is what proofs/verus/src/capsules/state.rs
        // calls `BusAddr`.  Sanity: distinct (b, d, f) → distinct words.
        let a = ConfigAddr {
            bus: 1,
            dev: 2,
            func: 3,
            offset: 0,
        }
        .pack_bdf();
        let b = ConfigAddr {
            bus: 1,
            dev: 2,
            func: 4,
            offset: 0,
        }
        .pack_bdf();
        let c = ConfigAddr {
            bus: 1,
            dev: 3,
            func: 0,
            offset: 0,
        }
        .pack_bdf();
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }
}

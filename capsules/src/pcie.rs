// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! `PCIe` enumeration capsule — the first concrete non-HIU capsule.
//!
//! Y4 needs `PCIe` enum to discover the `WaveTensor` accelerator board on
//! the host bus.  The walk is the standard recursive scan — every
//! (bus, device, function) is probed once and bridges trigger a
//! recursive descent into the secondary bus.
//!
//! Spec correspondence (`proofs/verus/src/capsules/pcie.rs`):
//!   * `P1 enum_deterministic` — the walk is a pure function of the
//!     [`ConfigSpace`] view (the abstraction's `read_u32` is itself
//!     deterministic).
//!   * `P2 enum_addrs_unique` — the walk visits each (b, d, f)
//!     exactly once; the result vec contains them in scan order.
//!   * `P3 enum_requires_bus_enumerator_cap` — `enumerate` requires
//!     the caller to present a [`BusEnumerator`] capability for the
//!     root bus.

use crate::config_space::{ConfigAddr, ConfigSpace};
use crate::error::Y4Error;
use crate::isolation::CapsulesState;
use crate::types::{CapsuleId, ResourceKind};

/// Maximum devices a single enum result vec can hold.  Sufficient for
/// any realistic Y4 host bus; production bumps via build-time const.
pub const MAX_ENUM_DEVICES: usize = 64;

/// One device produced by [`PcieEnumerator::enumerate`].  `PCIe` header
/// fields read directly from config space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcieDevice {
    /// Packed `(bus << 16) | (dev << 11) | (fn << 8)` — matches
    /// `proofs/verus/src/capsules/state.rs::BusAddr`.
    pub addr: u32,
    /// 16-bit vendor id from offset 0x00.
    pub vendor: u16,
    /// 16-bit device id from offset 0x02.
    pub device: u16,
    /// 24-bit class code from offset 0x09–0x0B.
    pub class: u32,
}

/// `PCIe` enumeration capsule.  Holds a reference to a `ConfigSpace`
/// implementor at construction time; each `enumerate` call performs
/// one fresh walk (no caching — see P1 determinism trade-off note).
pub struct PcieEnumerator<'a, C: ConfigSpace> {
    cs: &'a C,
}

impl<'a, C: ConfigSpace> PcieEnumerator<'a, C> {
    /// Wrap a config-space reader.
    pub fn new(cs: &'a C) -> Self {
        Self { cs }
    }

    /// Walk the bus tree starting at bus 0.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] when `caller` does not hold a
    /// [`ResourceKind::BusEnumerator`] cap (P3).
    /// [`Y4Error::NoMemory`] when more than `MAX_ENUM_DEVICES`
    /// devices are present.
    pub fn enumerate(
        &self,
        state: &CapsulesState,
        caller: CapsuleId,
    ) -> Result<heapless::Vec<PcieDevice, MAX_ENUM_DEVICES>, Y4Error> {
        // P3: cap check — caller must hold a BusEnumerator token.
        let capsule = state.capsule(caller).ok_or(Y4Error::BadCap)?;
        let has_cap = capsule.cap_set.iter().any(|t_id| {
            state
                .token(*t_id)
                .is_some_and(|t| t.kind == ResourceKind::BusEnumerator)
        });
        if !has_cap {
            return Err(Y4Error::BadCap);
        }
        let mut out: heapless::Vec<PcieDevice, MAX_ENUM_DEVICES> = heapless::Vec::new();
        // Track visited (bus, dev, fn) tuples to enforce P2 even
        // under malformed bridge tables that loop the bus graph.
        let mut visited_bdf: heapless::Vec<u32, MAX_ENUM_DEVICES> = heapless::Vec::new();
        self.walk_bus(0, &mut out, &mut visited_bdf)?;
        Ok(out)
    }

    fn walk_bus(
        &self,
        bus: u8,
        out: &mut heapless::Vec<PcieDevice, MAX_ENUM_DEVICES>,
        visited: &mut heapless::Vec<u32, MAX_ENUM_DEVICES>,
    ) -> Result<(), Y4Error> {
        for dev in 0..32u8 {
            // Probe function 0 first; if it's absent, skip the device.
            let vid = self.cs.read_u16(bus, dev, 0, 0x00);
            if vid == 0xFFFF {
                continue;
            }
            let header_type = self.cs.read_u8(bus, dev, 0, 0x0E);
            let multi_function = (header_type & 0x80) != 0;
            let max_func = if multi_function { 8u8 } else { 1u8 };
            for func in 0..max_func {
                self.probe_function(bus, dev, func, out, visited)?;
            }
        }
        Ok(())
    }

    fn probe_function(
        &self,
        bus: u8,
        dev: u8,
        func: u8,
        out: &mut heapless::Vec<PcieDevice, MAX_ENUM_DEVICES>,
        visited: &mut heapless::Vec<u32, MAX_ENUM_DEVICES>,
    ) -> Result<(), Y4Error> {
        let vendor = self.cs.read_u16(bus, dev, func, 0x00);
        if vendor == 0xFFFF {
            return Ok(());
        }
        let bdf = ConfigAddr {
            bus,
            dev,
            func,
            offset: 0,
        }
        .pack_bdf();
        if visited.contains(&bdf) {
            return Ok(()); // P2: already visited.
        }
        visited.push(bdf).map_err(|_| Y4Error::NoMemory)?;

        let device = self.cs.read_u16(bus, dev, func, 0x02);
        // class code lives at offset 0x09–0x0B; pull the dword at
        // offset 0x08 and shift off the revision byte.
        let class_dword = self.cs.read_u32(ConfigAddr {
            bus,
            dev,
            func,
            offset: 2,
        });
        let class = (class_dword >> 8) & 0x00FF_FFFF;
        let header_type = self.cs.read_u8(bus, dev, func, 0x0E);
        let raw_type = header_type & 0x7F;

        out.push(PcieDevice {
            addr: bdf,
            vendor,
            device,
            class,
        })
        .map_err(|_| Y4Error::NoMemory)?;

        // PCI-PCI bridge → recurse into secondary bus.
        if raw_type == 0x01 {
            let secondary = self.cs.read_u8(bus, dev, func, 0x19);
            // Guard against degenerate bridges that point at their own
            // bus — would otherwise infinite-recurse.
            if secondary != bus {
                self.walk_bus(secondary, out, visited)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config_space::{MockConfigSpace, MockDevice};
    use crate::isolation::CapsulesState;
    use crate::types::ResourceKind;

    fn cap_holding_enumerator() -> (CapsulesState, CapsuleId) {
        let mut s = CapsulesState::new();
        s.register_capsule(1).unwrap();
        s.mint_into(100, 1, ResourceKind::BusEnumerator, 0).unwrap();
        (s, 1)
    }

    fn cap_without_enumerator() -> (CapsulesState, CapsuleId) {
        let mut s = CapsulesState::new();
        s.register_capsule(2).unwrap();
        s.mint_into(101, 2, ResourceKind::Mmio, 0xABCD).unwrap();
        (s, 2)
    }

    fn single_device_topology() -> MockConfigSpace {
        let mut cs = MockConfigSpace::new();
        cs.add(MockDevice {
            bus: 0,
            dev: 1,
            func: 0,
            vendor_id: 0x10EC,
            device_id: 0x8168,     // Realtek NIC
            class_code: 0x02_0000, // Network controller
            header_type: 0x00,
            secondary_bus: 0,
        })
        .unwrap();
        cs
    }

    fn bridged_topology() -> MockConfigSpace {
        // Bus 0 has a bridge at 0:1.0 routing to bus 1.
        // Bus 1 has the WaveTensor accelerator at 1:0.0.
        let mut cs = MockConfigSpace::new();
        cs.add(MockDevice {
            bus: 0,
            dev: 1,
            func: 0,
            vendor_id: 0x8086,
            device_id: 0xAAAA,     // Intel bridge
            class_code: 0x06_0400, // PCI-PCI bridge
            header_type: 0x01,
            secondary_bus: 1,
        })
        .unwrap();
        cs.add(MockDevice {
            bus: 1,
            dev: 0,
            func: 0,
            vendor_id: 0x1D6B,
            device_id: 0xCAFE,     // WaveTensor
            class_code: 0x12_0000, // Processing accelerator
            header_type: 0x00,
            secondary_bus: 0,
        })
        .unwrap();
        cs
    }

    #[test]
    fn p3_enum_without_cap_rejected() {
        let cs = single_device_topology();
        let (state, caller) = cap_without_enumerator();
        let e = PcieEnumerator::new(&cs);
        assert_eq!(e.enumerate(&state, caller), Err(Y4Error::BadCap));
    }

    #[test]
    fn p3_unknown_caller_rejected() {
        let cs = single_device_topology();
        let state = CapsulesState::new();
        let e = PcieEnumerator::new(&cs);
        assert_eq!(e.enumerate(&state, 99), Err(Y4Error::BadCap));
    }

    #[test]
    fn enumerate_single_device() {
        let cs = single_device_topology();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let devs = e.enumerate(&state, caller).unwrap();
        assert_eq!(devs.len(), 1);
        assert_eq!(devs[0].vendor, 0x10EC);
        assert_eq!(devs[0].device, 0x8168);
        assert_eq!(devs[0].class, 0x02_0000);
    }

    #[test]
    fn p2_addrs_unique() {
        let cs = bridged_topology();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let devs = e.enumerate(&state, caller).unwrap();
        assert_eq!(devs.len(), 2);
        for i in 0..devs.len() {
            for j in (i + 1)..devs.len() {
                assert_ne!(devs[i].addr, devs[j].addr);
            }
        }
    }

    #[test]
    fn p1_deterministic_across_calls() {
        // Same (state, config-space) ⇒ identical result on every call.
        let cs = bridged_topology();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let r1 = e.enumerate(&state, caller).unwrap();
        let r2 = e.enumerate(&state, caller).unwrap();
        assert_eq!(r1, r2);
    }

    #[test]
    fn bridge_recursion_descends_to_secondary_bus() {
        let cs = bridged_topology();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let devs = e.enumerate(&state, caller).unwrap();
        let wt = devs
            .iter()
            .find(|d| d.vendor == 0x1D6B)
            .expect("WaveTensor not enumerated");
        assert_eq!(wt.device, 0xCAFE);
        assert_eq!(wt.class, 0x12_0000);
    }

    #[test]
    fn self_loop_bridge_does_not_infinite_recurse() {
        // A malformed bridge that reports its own bus as secondary
        // must not trip the walk into an infinite loop.
        let mut cs = MockConfigSpace::new();
        cs.add(MockDevice {
            bus: 0,
            dev: 2,
            func: 0,
            vendor_id: 0xDEAD,
            device_id: 0xBEEF,
            class_code: 0x06_0400,
            header_type: 0x01,
            secondary_bus: 0, // points at self
        })
        .unwrap();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let devs = e.enumerate(&state, caller).unwrap();
        assert_eq!(devs.len(), 1);
    }

    #[test]
    fn multi_function_device_enumerated_per_function() {
        let mut cs = MockConfigSpace::new();
        // dev 5 is multi-function (header_type bit 7 set on func 0).
        cs.add(MockDevice {
            bus: 0,
            dev: 5,
            func: 0,
            vendor_id: 0x1234,
            device_id: 0xAAAA,
            class_code: 0x04_0000,
            header_type: 0x80, // multi-function bit + endpoint
            secondary_bus: 0,
        })
        .unwrap();
        cs.add(MockDevice {
            bus: 0,
            dev: 5,
            func: 1,
            vendor_id: 0x1234,
            device_id: 0xBBBB,
            class_code: 0x04_0000,
            header_type: 0x00,
            secondary_bus: 0,
        })
        .unwrap();
        let (state, caller) = cap_holding_enumerator();
        let e = PcieEnumerator::new(&cs);
        let devs = e.enumerate(&state, caller).unwrap();
        assert_eq!(devs.len(), 2);
    }
}

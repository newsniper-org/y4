// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 윤병익 (BYUNG-IK YEUN) and Y4 contributors

//! Capsule isolation invariants — runtime predicates corresponding to
//! C1 / C2 / C3 in `proofs/verus/src/capsules/isolation.rs`.

use crate::error::Y4Error;
use crate::types::{CapToken, CapTokenId, Capsule, CapsuleId, ResourceKind};

/// Whole-subsystem state — capsule table + token table.  Bounded
/// capacity matches the typical Phase B capsule count; `kernel/`
/// raises this with build-time const propagation when it lands.
pub const MAX_CAPSULES: usize = 32;

/// Maximum tokens tracked across all capsules.
pub const MAX_TOKENS: usize = 256;

/// Whole capsule subsystem state.
#[derive(Debug, Default)]
pub struct CapsulesState {
    /// (`CapsuleId`, `Capsule`) entries.  Insertion order; lookup is O(n)
    /// for the small `MAX_CAPSULES` we ship.
    pub capsules: heapless::LinearMap<CapsuleId, Capsule, MAX_CAPSULES>,
    /// (`CapTokenId`, `CapToken`) entries.
    pub tokens: heapless::LinearMap<CapTokenId, CapToken, MAX_TOKENS>,
}

impl CapsulesState {
    /// Empty state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new capsule.
    ///
    /// # Errors
    /// [`Y4Error::NoMemory`] when `MAX_CAPSULES` is exhausted.
    pub fn register_capsule(&mut self, id: CapsuleId) -> Result<(), Y4Error> {
        self.capsules
            .insert(id, Capsule::new())
            .map_err(|_| Y4Error::NoMemory)?;
        Ok(())
    }

    /// Mint a token into a registered capsule.  This is the only path
    /// that adds to either table — capsules themselves cannot reach
    /// this method (it lives in the kernel boot path), preserving C2.
    ///
    /// # Errors
    /// [`Y4Error::BadCap`] if `holder` is not registered;
    /// [`Y4Error::NoMemory`] if either table is full.
    pub fn mint_into(
        &mut self,
        token_id: CapTokenId,
        holder: CapsuleId,
        kind: ResourceKind,
        resource: u32,
    ) -> Result<(), Y4Error> {
        let capsule = self.capsules.get_mut(&holder).ok_or(Y4Error::BadCap)?;
        capsule
            .cap_set
            .push(token_id)
            .map_err(|_| Y4Error::NoMemory)?;
        self.tokens
            .insert(
                token_id,
                CapToken {
                    holder,
                    resource,
                    kind,
                },
            )
            .map_err(|_| Y4Error::NoMemory)?;
        Ok(())
    }

    /// Look up a token by id.
    #[must_use]
    pub fn token(&self, id: CapTokenId) -> Option<&CapToken> {
        self.tokens.get(&id)
    }

    /// Look up a capsule by id.
    #[must_use]
    pub fn capsule(&self, id: CapsuleId) -> Option<&Capsule> {
        self.capsules.get(&id)
    }

    /// Whole-state well-formedness — runs C1 + C2 + C3 in one shot.
    /// Cheap (O(N²) on small N) so callers can sprinkle it in tests
    /// or in PR-gate audits without measurable cost.
    #[must_use]
    pub fn well_formed(&self) -> bool {
        c1_holds(self) && c2_holds(self) && c3_holds(self)
    }
}

/// **C1 — token unique owner.**  Map structure already guarantees
/// `token_id → CapToken`'s functional shape; here we additionally
/// check that every recorded token's holder actually exists.
#[must_use]
pub fn c1_holds(s: &CapsulesState) -> bool {
    s.tokens
        .iter()
        .all(|(_, t)| s.capsules.contains_key(&t.holder))
}

/// **C2 — no capsule can mint.**  Every token recorded as belonging
/// to a capsule's `cap_set` agrees with the global token table's
/// holder field.  A capsule "minting" a fake token would violate this.
#[must_use]
pub fn c2_holds(s: &CapsulesState) -> bool {
    s.capsules.iter().all(|(c_id, capsule)| {
        capsule
            .cap_set
            .iter()
            .all(|t_id| s.tokens.get(t_id).is_some_and(|t| t.holder == *c_id))
    })
}

/// **C3 — resource disjoint or explicit share.**  Two distinct
/// capsules may both reference the same resource only via two
/// distinct token ids (each minted as a separate share by the kernel).
#[must_use]
pub fn c3_holds(s: &CapsulesState) -> bool {
    for (id_a, cap_a) in &s.capsules {
        for (id_b, cap_b) in &s.capsules {
            if id_a == id_b {
                continue;
            }
            for ta in &cap_a.cap_set {
                for tb in &cap_b.cap_set {
                    if let (Some(token_a), Some(token_b)) = (s.tokens.get(ta), s.tokens.get(tb))
                        && token_a.resource == token_b.resource
                        && ta == tb
                    {
                        // Same resource, same token id, two distinct
                        // holders — violates C3.
                        return false;
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build() -> CapsulesState {
        let mut s = CapsulesState::new();
        s.register_capsule(1).unwrap();
        s.register_capsule(2).unwrap();
        s
    }

    #[test]
    fn empty_state_well_formed() {
        let s = CapsulesState::new();
        assert!(s.well_formed());
    }

    #[test]
    fn fresh_registration_well_formed() {
        let s = build();
        assert!(s.well_formed());
    }

    #[test]
    fn mint_into_unknown_capsule_fails() {
        let mut s = CapsulesState::new();
        assert_eq!(
            s.mint_into(100, 99, ResourceKind::Mmio, 0xABCD),
            Err(Y4Error::BadCap)
        );
    }

    #[test]
    fn c1_c2_after_mint() {
        let mut s = build();
        s.mint_into(10, 1, ResourceKind::Mmio, 0x1000).unwrap();
        s.mint_into(11, 2, ResourceKind::Irq, 5).unwrap();
        assert!(c1_holds(&s));
        assert!(c2_holds(&s));
        assert!(c3_holds(&s));
    }

    #[test]
    fn c3_explicit_share_two_tokens() {
        // Two capsules can each get a *distinct* token for the same
        // PCIe device — kernel mints two shares.  C3 holds.
        let mut s = build();
        s.mint_into(20, 1, ResourceKind::Mmio, 0xCAFE).unwrap();
        s.mint_into(21, 2, ResourceKind::Mmio, 0xCAFE).unwrap();
        assert!(c3_holds(&s));
    }
}

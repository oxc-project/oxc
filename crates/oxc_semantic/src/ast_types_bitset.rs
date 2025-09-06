use oxc_ast::{AstType, ast_kind::AST_TYPE_MAX};

const USIZE_BITS: usize = usize::BITS as usize;

/// Number of bytes required for bit set which can represent all [`AstType`]s.
// Need to add plus one here because 0 is a possible value, but requires at least one bit to represent it.
const NUM_USIZES: usize = (AST_TYPE_MAX as usize + 1).div_ceil(USIZE_BITS);

/// Bit set with a bit for each [`AstType`].
#[derive(Debug, Clone)]
pub struct AstTypesBitset([usize; NUM_USIZES]);

impl AstTypesBitset {
    /// Create empty [`AstTypesBitset`] with no bits set.
    pub const fn new() -> Self {
        Self([0; NUM_USIZES])
    }

    /// Create a new [`AstTypesBitset`] from a slice of [`AstType`].
    pub fn from_types(types: &[AstType]) -> Self {
        let mut bitset = Self::new();
        for &ty in types {
            bitset.set(ty);
        }
        bitset
    }

    /// Returns `true` if bit is set for provided [`AstType`].
    pub const fn has(&self, ty: AstType) -> bool {
        let (index, mask) = Self::index_and_mask(ty);
        (self.0[index] & mask) != 0
    }

    /// Set bit for provided [`AstType`].
    pub const fn set(&mut self, ty: AstType) {
        let (index, mask) = Self::index_and_mask(ty);
        self.0[index] |= mask;
    }

    /// Returns `true` if any bit is set in both `self` and `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        let mut intersection = 0;
        for (&a, &b) in self.0.iter().zip(other.0.iter()) {
            intersection |= a & b;
        }
        intersection != 0
    }

    /// Returns `true` if all bits in `other` are set in `self`.
    pub fn contains(&self, other: &Self) -> bool {
        let mut mismatches = 0;
        for (&a, &b) in self.0.iter().zip(other.0.iter()) {
            let set_in_both = a & b;
            // 0 if `set_in_both == b`
            let mismatch = set_in_both ^ b;
            mismatches |= mismatch;
        }
        mismatches == 0
    }

    /// Get index and mask for an [`AstType`].
    /// Returned `index` is guaranteed not to be out of bounds of the array.
    const fn index_and_mask(ty: AstType) -> (usize, usize) {
        let n = ty as usize;
        let index = n / USIZE_BITS;
        let mask = 1usize << (n % USIZE_BITS);
        (index, mask)
    }
}

impl Default for AstTypesBitset {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_ast::AstType;

    #[test]
    fn empty_bitset_has_no_bits_and_contains_empty() {
        let bs = AstTypesBitset::new();
        assert!(!bs.has(AstType::Program));
        let other = AstTypesBitset::new();
        assert!(bs.contains(&other), "Empty bitset should contain empty bitset");
        assert!(!bs.intersects(&other));
    }

    #[test]
    fn intersects_and_contains() {
        let mut a = AstTypesBitset::from_types(&[AstType::Program, AstType::AssignmentPattern]);
        let b = AstTypesBitset::from_types(&[AstType::TSTupleType]);
        assert!(!a.intersects(&b));
        a.set(AstType::TSTupleType);
        assert!(a.intersects(&b));

        let c = AstTypesBitset::from_types(&[AstType::Program]);
        assert!(a.contains(&c));
        assert!(!c.contains(&a));

        // a should contain union of subset bits
        let subset = AstTypesBitset::from_types(&[AstType::Program, AstType::AssignmentPattern]);
        assert!(a.contains(&subset));
        // subset does not contain a (missing TSTupleType)
        assert!(!subset.contains(&a));
    }

    #[test]
    fn contains_empty_is_true() {
        let non_empty = AstTypesBitset::from_types(&[AstType::IdentifierName]);
        let empty = AstTypesBitset::new();
        assert!(non_empty.contains(&empty));
    }
}

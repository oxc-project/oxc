use oxc_ast::{AstType, ast_kind::AST_TYPE_MAX};

const USIZE_BITS: usize = usize::BITS as usize;

/// Number of bytes required for bit set which can represent all [`AstType`]s.
// Need to add plus one here because 0 is a possible value, but requires at least one bit to represent it.
const NUM_USIZES: usize = ((AST_TYPE_MAX + 1) as usize).div_ceil(USIZE_BITS);

/// Bit set with a bit for each [`AstType`].
#[derive(Debug, Clone)]
pub struct AstTypesBitset([usize; NUM_USIZES]);

impl AstTypesBitset {
    /// Create empty [`AstTypesBitset`] with no bits set.
    pub const fn new() -> Self {
        Self([0; NUM_USIZES])
    }

    pub const fn from_types(types: &[AstType]) -> Self {
        let mut bitset = Self::new();
        let mut i = 0;
        while i < types.len() {
            bitset.set(types[i]);
            i += 1;
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
        for (&a, &b) in self.0.iter().zip(other.0.iter()) {
            if a & b != b {
                return false;
            }
        }
        true
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
        let mut a = AstTypesBitset::new();
        a.set(AstType::Program);
        a.set(AstType::AssignmentPattern);
        let mut b = AstTypesBitset::new();
        b.set(AstType::TSTupleType);
        assert!(!a.intersects(&b));
        a.set(AstType::TSTupleType);
        assert!(a.intersects(&b));

        let mut c = AstTypesBitset::new();
        c.set(AstType::Program);
        assert!(a.contains(&c));
        assert!(!c.contains(&a));

        // a should contain union of subset bits
        let mut subset = AstTypesBitset::new();
        subset.set(AstType::Program);
        subset.set(AstType::AssignmentPattern);
        assert!(a.contains(&subset));
        // subset does not contain a (missing TSTupleType)
        assert!(!subset.contains(&a));
    }

    #[test]
    fn contains_empty_is_true() {
        let mut non_empty = AstTypesBitset::new();
        non_empty.set(AstType::IdentifierName);
        let empty = AstTypesBitset::new();
        assert!(non_empty.contains(&empty));
    }
}

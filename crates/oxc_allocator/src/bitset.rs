use std::fmt::{Debug, Display};

use crate::{Allocator, CloneIn, Vec};

/// A bitset allocated in an arena.
#[derive(PartialEq, Eq, Hash)]
pub struct BitSet<'alloc> {
    entries: Vec<'alloc, u8>,
}

impl<'alloc> BitSet<'alloc> {
    /// Create new [`BitSet`] with size `max_bit_count`, in the specified allocator.
    pub fn new_in(max_bit_count: usize, allocator: &'alloc Allocator) -> Self {
        Self {
            entries: Vec::from_iter_in(
                std::iter::repeat_n(0, max_bit_count.div_ceil(8)),
                allocator,
            ),
        }
    }

    /// Returns `true` if the bit at the given position is set.
    pub fn has_bit(&self, bit: usize) -> bool {
        (self.entries[bit / 8] & (1 << (bit & 7))) != 0
    }

    /// Set the bit at the given position.
    pub fn set_bit(&mut self, bit: usize) {
        self.entries[bit / 8] |= 1 << (bit & 7);
    }
}

impl Display for BitSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // using little endian representation
        // e.g. 256
        // 00000001_00000000
        // ^               ^
        // msb             lsb
        let mut iter = self.entries.iter().rev();
        if let Some(first) = iter.next() {
            f.write_str(&format!("{first:08b}"))?;
        }
        for e in iter {
            f.write_str(&format!("_{e:08b}"))?;
        }
        Ok(())
    }
}

impl Debug for BitSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("BitSet").field(&self.to_string()).finish()
    }
}

impl<'allocator> CloneIn<'allocator> for BitSet<'allocator> {
    type Cloned = Self;

    fn clone_in(&self, allocator: &'allocator Allocator) -> Self {
        Self { entries: self.entries.clone_in(allocator) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(1, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");

        let mut bs = BitSet::new_in(9, &allocator);
        assert_eq!(bs.to_string(), "00000000_00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "00000000_10000011");
        bs.set_bit(8);
        assert_eq!(bs.to_string(), "00000001_10000011");
        bs.set_bit(15);
        assert_eq!(bs.to_string(), "10000001_10000011");
    }

    #[test]
    fn union() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(9, &allocator);
        assert_eq!(bs.to_string(), "00000000_00000000");
        let mut bs2 = bs.clone_in(&allocator);
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "00000000_10000011");
        bs2.set_bit(8);
        bs2.set_bit(15);
        assert_eq!(bs2.to_string(), "10000001_00000000");
    }
}

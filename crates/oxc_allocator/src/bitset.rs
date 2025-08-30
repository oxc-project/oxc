use std::{
    fmt::{self, Debug, Display},
    iter,
};

use crate::{Allocator, CloneIn, Vec};

const USIZE_BITS: usize = usize::BITS as usize;

/// A bitset allocated in an arena.
#[derive(PartialEq, Eq, Hash)]
pub struct BitSet<'alloc> {
    entries: Vec<'alloc, usize>,
}

impl<'alloc> BitSet<'alloc> {
    /// Create new [`BitSet`] with size `max_bit_count`, in the specified allocator.
    pub fn new_in(max_bit_count: usize, allocator: &'alloc Allocator) -> Self {
        Self {
            entries: Vec::from_iter_in(
                iter::repeat_n(0, max_bit_count.div_ceil(USIZE_BITS)),
                allocator,
            ),
        }
    }

    /// Returns `true` if the bit at the given position is set.
    pub fn has_bit(&self, bit: usize) -> bool {
        (self.entries[bit / USIZE_BITS] & (1 << (bit % USIZE_BITS))) != 0
    }

    /// Set the bit at the given position.
    pub fn set_bit(&mut self, bit: usize) {
        self.entries[bit / USIZE_BITS] |= 1 << (bit % USIZE_BITS);
    }
}

impl Display for BitSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Using big endian representation
        // e.g. 256:
        // 00000001_00000000
        // ^               ^
        // msb             lsb

        if self.entries.is_empty() {
            return Ok(());
        }

        let mut usizes = self.entries.iter().rev();

        // Skip leading zeros
        while usizes.clone().next() == Some(&0) {
            usizes.next();
        }

        let Some(highest_usize) = usizes.next() else {
            // All zeros - print 8 bits
            return f.write_str("00000000");
        };

        // Print highest `usize`
        let bytes = highest_usize.to_ne_bytes();
        #[cfg(target_endian = "little")]
        let mut bytes = bytes.iter().rev();
        #[cfg(target_endian = "big")]
        let mut bytes = bytes.iter();

        // Skip leading zeros
        while bytes.clone().next() == Some(&0) {
            bytes.next();
        }

        let highest_byte = bytes.next().unwrap();
        f.write_str(&format!("{highest_byte:08b}"))?;

        for byte in bytes {
            f.write_str(&format!("_{byte:08b}"))?;
        }

        // Print remaining `usize`s without skipping any bytes
        for lower_usize in usizes {
            let bytes = lower_usize.to_ne_bytes();
            #[cfg(target_endian = "little")]
            let bytes = bytes.iter().rev();
            #[cfg(target_endian = "big")]
            let bytes = bytes.iter();

            for byte in bytes {
                f.write_str(&format!("_{byte:08b}"))?;
            }
        }

        Ok(())
    }
}

impl Debug for BitSet<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        let mut bs = BitSet::new_in(64, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs.set_bit(9);
        assert_eq!(bs.to_string(), "00000010_10000011");
        bs.set_bit(63);
        assert_eq!(
            bs.to_string(),
            "10000000_00000000_00000000_00000000_00000000_00000000_00000010_10000011"
        );

        let mut bs = BitSet::new_in(65, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs.set_bit(8);
        assert_eq!(bs.to_string(), "00000001_10000011");
        bs.set_bit(15);
        assert_eq!(bs.to_string(), "10000001_10000011");
        bs.set_bit(63);
        assert_eq!(
            bs.to_string(),
            "10000000_00000000_00000000_00000000_00000000_00000000_10000001_10000011"
        );
        bs.set_bit(64);
        assert_eq!(
            bs.to_string(),
            "00000001_10000000_00000000_00000000_00000000_00000000_00000000_10000001_10000011"
        );
    }

    #[test]
    fn union() {
        let allocator = Allocator::default();
        let mut bs = BitSet::new_in(16, &allocator);
        assert_eq!(bs.to_string(), "00000000");
        let mut bs2 = bs.clone_in(&allocator);
        bs.set_bit(0);
        bs.set_bit(1);
        bs.set_bit(7);
        assert_eq!(bs.to_string(), "10000011");
        bs2.set_bit(8);
        bs2.set_bit(15);
        assert_eq!(bs2.to_string(), "10000001_00000000");
    }
}

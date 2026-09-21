#[repr(C, align(64))]
pub struct PairLuts {
    pub lut0z: [[u8; 8]; 256],
    pub lutpad: [[u8; 32]; 256],
}

/// Ensure `lutpad` field is aligned on a 64-byte boundary.
const _: () = assert!(size_of::<[[u8; 8]; 256]>().is_multiple_of(64));

impl PairLuts {
    pub(super) fn new() -> Self {
        Self { lut0z: [[0; 8]; 256], lutpad: [[0; 32]; 256] }
    }

    pub(super) fn build(&mut self) {
        for m in 0..256usize {
            let mut k = 0usize;
            for bit in 0..8usize {
                if (m >> bit) & 1 != 0 {
                    self.lut0z[m][k] = bit as u8;
                    self.lutpad[m][8 + k] = (bit + 8) as u8;
                    k += 1;
                }
            }

            for j in k..8 {
                self.lutpad[m][8 + j] = 0x80;
            }
        }
    }
}

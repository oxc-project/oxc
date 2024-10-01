/// Calculates the Shannon entropy of a byte string.
///
/// Implementation borrowed from [Rosetta Code](https://rosettacode.org/wiki/Entropy#Rust).
///
/// see: [Entropy (Wikipedial)](https://en.wikipedia.org/wiki/Entropy_(information_theory))
#[allow(clippy::cast_precision_loss)]
pub(crate) fn entropy<S: AsRef<[u8]>>(string: S) -> f32 {
    let mut histogram = [0u32; 256];
    let bytes = string.as_ref();
    // we don't care if this is truncated
    let len = bytes.len() as f32;

    for &b in bytes {
        histogram[b as usize] += 1;
    }

    histogram
        .iter()
        .copied()
        .filter(|&h| h != 0)
        .map(|h| h as f32 / len) // we don't care if this is truncated
        .map(|ratio| -ratio * ratio.log2())
        .sum()
}

pub(crate) trait Entropy {
    /// Calculates the Shannon entropy of a byte string.
    ///
    /// Implementation borrowed from [Rosetta Code](https://rosettacode.org/wiki/Entropy#Rust).
    ///
    /// see: [Entropy (Wikipedial)](https://en.wikipedia.org/wiki/Entropy_(information_theory))
    fn entropy(&self) -> f32;
}

impl<S> Entropy for S
where
    S: AsRef<[u8]>,
{
    fn entropy(&self) -> f32 {
        entropy(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_entropy() {
        let test_cases = vec![
            ("hello world", "hello world".entropy()),
            ("hello world", b"hello world".entropy()),
            ("hello world", String::from("hello world").entropy()),
            ("hello world", 2.845_351_2),
        ];

        for (input, expected) in test_cases {
            let actual = entropy(input);
            assert!(
                (actual - expected).abs() < f32::EPSILON,
                "expected entropy({input}) to be {expected}, got {actual}"
            );
        }
    }
}

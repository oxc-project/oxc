//! Levenshtein edit distance calculation for suggesting similar names.

/// Returns the Levenshtein edit distance between `a` and `b`.
///
/// Uses a two-row dynamic programming algorithm to keep memory usage small.
pub fn min_edit_distance(a: &str, b: &str) -> usize {
    if a.len() < b.len() {
        return min_edit_distance(b, a);
    }

    let b_chars: Vec<char> = b.chars().collect();

    let n = b_chars.len();
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = Vec::with_capacity(n + 1);
    for (i, ca) in a.chars().enumerate() {
        curr.clear();
        curr.push(i + 1);
        for (j, &cb) in b_chars.iter().enumerate() {
            curr.push((prev[j] + usize::from(ca != cb)).min(prev[j + 1] + 1).min(curr[j] + 1));
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Returns the closest candidate to `needle` within `threshold`, or `None`.
///
/// Exact matches return `None` to indicate "not a typo".
pub fn best_match<'a>(
    needle: &str,
    candidates: impl IntoIterator<Item = &'a str>,
    threshold: usize,
) -> Option<&'a str> {
    let mut best: Option<(&'a str, usize)> = None;
    let needle_len = needle.chars().count();

    for candidate in candidates {
        // The distance counts Unicode scalar values, not UTF-8 bytes.
        // No need to calculate it if the character length difference exceeds the threshold.
        if candidate.chars().count().abs_diff(needle_len) > threshold {
            continue;
        }
        let distance = min_edit_distance(candidate, needle);
        if distance == 0 {
            return None;
        }
        if distance <= threshold {
            match best {
                Some((_, best_distance)) if distance >= best_distance => {}
                _ => best = Some((candidate, distance)),
            }
        }
    }

    best.map(|(candidate, _)| candidate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_edit_distance() {
        assert_eq!(min_edit_distance("", ""), 0);
        assert_eq!(min_edit_distance("a", "a"), 0);
        assert_eq!(min_edit_distance("abc", "abc"), 0);
        assert_eq!(min_edit_distance("", "abc"), 3);
        assert_eq!(min_edit_distance("abc", ""), 3);
        assert_eq!(min_edit_distance("abc", "def"), 3);
        assert_eq!(min_edit_distance("sitting", "kitten"), 3);
        assert_eq!(min_edit_distance("a", "𐐀"), 1);
        assert_eq!(min_edit_distance("a𐐀", "a"), 1);
    }

    #[test]
    fn test_best_match() {
        let candidates = vec!["apple", "banana", "cherry"];

        // Exact match returns None
        assert_eq!(best_match("apple", candidates.clone(), 2), None);

        // Close match within threshold
        assert_eq!(best_match("aple", candidates.clone(), 2), Some("apple"));
        assert_eq!(best_match("banan", candidates.clone(), 2), Some("banana"));

        // No match within threshold
        assert_eq!(best_match("xyz", candidates.clone(), 2), None);

        // Empty candidates
        let empty: Vec<&str> = vec![];
        assert_eq!(best_match("test", empty, 2), None);
    }

    #[test]
    fn test_best_match_unicode() {
        assert_eq!(best_match("a", ["𐐀"], 1), Some("𐐀"));
        assert_eq!(best_match("𐐀", ["a"], 1), Some("a"));
        assert_eq!(best_match("a", ["a𐐀"], 1), Some("a𐐀"));
        assert_eq!(best_match("a𐐀", ["a"], 1), Some("a"));
        assert_eq!(best_match("a", ["é"], 0), None);
        assert_eq!(best_match("𐐀", ["𐐀"], 1), None);
        assert_eq!(best_match("a", ["𐐀𐐀𐐀"], 1), None);
    }
}

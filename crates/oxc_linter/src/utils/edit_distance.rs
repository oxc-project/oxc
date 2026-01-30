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

    for candidate in candidates {
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

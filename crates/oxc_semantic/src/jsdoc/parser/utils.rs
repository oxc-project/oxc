// For now, just returns the most outer braces
pub fn find_type_range(s: &str) -> Option<(usize, usize)> {
    let mut start = None;
    let mut brace_count = 0;
    for (idx, ch) in s.char_indices() {
        match ch {
            '{' => {
                brace_count += 1;

                if start.is_none() {
                    start = Some(idx);
                }
            }
            '}' => {
                brace_count -= 1;

                if brace_count == 0 {
                    if let Some(start) = start {
                        return Some((start, idx + 1));
                    }
                }
            }
            _ => {}
        }
    }
    None
}

// Like a token but whitespace may appear inside of optional type syntax
// e.g. `[foo = 1]`, `[bar="here inside of string"]`, `[ baz = [ "a b", "c" ] ]`
pub fn find_type_name_range(s: &str) -> Option<(usize, usize)> {
    // Not optional type syntax
    if !s.trim_start().starts_with('[') {
        return find_token_range(s);
    }

    let mut bracket = 0;
    let mut start = None;
    for (idx, ch) in s.char_indices() {
        if ch.is_whitespace() {
            if bracket != 0 {
                continue;
            }

            if let Some(start) = start {
                return Some((start, idx));
            }
        } else {
            if ch == '[' {
                bracket += 1;
            }
            if ch == ']' {
                bracket -= 1;
            }

            if start.is_none() {
                start = Some(idx);
            }
        }
    }

    // Everything is a token
    if let Some(start) = start {
        return Some((start, s.len()));
    }

    None
}

// Find inline token string as range
pub fn find_token_range(s: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (idx, ch) in s.char_indices() {
        // `{` may appear just after `@kind{type}`
        // Other syntax characters also can be splitter...?
        if ch.is_whitespace() || ch == '{' {
            if let Some(start) = start {
                return Some((start, idx));
            }
        } else if start.is_none() {
            start = Some(idx);
        }
    }

    // Everything is a token
    if let Some(start) = start {
        return Some((start, s.len()));
    }

    None
}

#[cfg(test)]
mod test {
    use super::{find_token_range, find_type_name_range, find_type_range};

    #[test]
    fn extract_type_part_range() {
        for (actual, expect) in [
            ("{t1}", Some("{t1}")),
            (" { t2 } ", Some("{ t2 }")),
            ("x{{ t3: string }}x", Some("{{ t3: string }}")),
            ("{t4} name", Some("{t4}")),
            (" {t5} ", Some("{t5}")),
            ("{t6 x", None),
            ("t7", None),
            ("{{t8}", None),
            ("", None),
            ("{[ true, false ]}", Some("{[ true, false ]}")),
            (
                "{{
t9a: string;
t9b: number;
}}",
                Some("{{\nt9a: string;\nt9b: number;\n}}"),
            ),
        ] {
            assert_eq!(find_type_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }

    #[test]
    fn extract_type_name_part_range() {
        for (actual, expect) in [
            ("", None),
            ("n1", Some("n1")),
            (" n2 ", Some("n2")),
            (" n3 n3", Some("n3")),
            ("[n4]\n", Some("[n4]")),
            ("[n5 = 1]", Some("[n5 = 1]")),
            ("  [n6 = [1,[2, [3]]]]  ", Some("[n6 = [1,[2, [3]]]]")),
            (r#"[n7 = "foo bar"]"#, Some(r#"[n7 = "foo bar"]"#)),
            ("n.n8", Some("n.n8")),
            ("n[].n9", Some("n[].n9")),
            (r#"[ n10 = ["{}", "[]"] ]"#, Some(r#"[ n10 = ["{}", "[]"] ]"#)),
            ("[n11... c11", Some("[n11... c11")),
            ("[n12[]\nc12", Some("[n12[]\nc12")),
            ("n12.n12", Some("n12.n12")),
            ("n13[].n13", Some("n13[].n13")),
        ] {
            assert_eq!(find_type_name_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }

    #[test]
    fn extract_token_part_range() {
        for (actual, expect) in [
            ("k1", Some("k1")),
            ("k2 x", Some("k2")),
            (" k3 ", Some("k3")),
            ("k4\ny", Some("k4")),
            ("", None),
            (" トークン5\n", Some("トークン5")),
            ("\nk6\nx", Some("k6")),
            ("k7{", Some("k7")),
        ] {
            assert_eq!(find_token_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }
}

// For now, just returns inside of most outer braces
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

// Find inline token string as range
pub fn find_token_range(s: &str) -> Option<(usize, usize)> {
    let mut start = None;
    for (idx, ch) in s.char_indices() {
        if ch.is_whitespace() {
            if let Some(start) = start {
                return Some((start, idx));
            }
        } else if start.is_none() {
            start = Some(idx);
        }
    }

    // Everything is a name
    if let Some(start) = start {
        return Some((start, s.len()));
    }

    None
}

#[cfg(test)]
mod test {
    use super::{find_token_range, find_type_range};

    //     #[test]
    //     fn trim_jsdoc_comments() {
    //         for (actual, expect) in [
    //             ("", ""),
    //             ("hello  ", "hello"),
    //             ("  * single line", "* single line"),
    //             (" * ", "*"),
    //             (" * * ", "* *"),
    //             ("***", "***"),
    //             (
    //                 "
    //   trim
    // ", "trim",
    //             ),
    //             (
    //                 "

    // ", "",
    //             ),
    //             (
    //                 "
    //                 *
    //                 *
    //                 ",
    //                 "",
    //             ),
    //             (
    //                 "
    //  * asterisk
    // ",
    //                 "asterisk",
    //             ),
    //             (
    //                 "
    //  * * li
    //  * * li
    // ",
    //                 "* li\n* li",
    //             ),
    //             (
    //                 "
    // * list
    // * list
    // ",
    //                 "list\nlist",
    //             ),
    //             (
    //                 "
    //  * * 1
    //  ** 2
    // ",
    //                 "* 1\n* 2",
    //             ),
    //             (
    //                 "
    // 1

    // 2

    // 3
    //             ",
    //                 "1\n2\n3",
    //             ),
    //         ] {
    //             assert_eq!(trim_comment(actual), expect);
    //         }
    //     }

    #[test]
    fn extract_type_part_range() {
        for (actual, expect) in [
            ("{t1}", Some("{t1}")),
            (" { t2 } ", Some("{ t2 }")),
            ("{{ t3: string }}", Some("{{ t3: string }}")),
            ("{t4} name", Some("{t4}")),
            (" {t5} ", Some("{t5}")),
            ("{t6 x", None),
            ("t7", None),
            ("{{t8}", None),
            ("", None),
            ("{[ true, false ]}", Some("{[ true, false ]}")),
        ] {
            assert_eq!(find_type_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }

    #[test]
    fn extract_token_part_range() {
        for (actual, expect) in [
            ("n1", Some("n1")),
            ("n2 x", Some("n2")),
            (" n3 ", Some("n3")),
            ("n4\ny", Some("n4")),
            ("", None),
            (" 名前5\n", Some("名前5")),
            ("\nn6\nx", Some("n6")),
        ] {
            assert_eq!(find_token_range(actual).map(|(s, e)| &actual[s..e]), expect);
        }
    }
}

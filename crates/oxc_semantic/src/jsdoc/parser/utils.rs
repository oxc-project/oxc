pub fn trim_multiline_comment(s: &str) -> String {
    s.trim()
        .split('\n')
        .map(|line| line.trim().trim_start_matches('*').trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod test {
    use super::trim_multiline_comment;

    #[test]
    fn trim_multiline_jsdoc_comments() {
        for (actual, expect) in [
            ("hello", "hello"),
            (
                "
  trim
", "trim",
            ),
            (
                "
 * asterisk
",
                "asterisk",
            ),
            (
                "
 * * li
 * * li
",
                "* li\n* li",
            ),
            (
                "
* list
* list
",
                "list\nlist",
            ),
            (
                "
1

2


3
            ",
                "1\n2\n3",
            ),
        ] {
            assert_eq!(trim_multiline_comment(actual), expect);
        }
    }
}

use super::utils;

// Since users use(invent!) any kind of tag and body, we can not enforce any specific format.
// Instead, we provide helper methods to parse the body.
//
// At first, I tried to handle common templates and parse it into specific struct like `JSDocParameterTag`.
// But I also found that some usecases like `eslint-plugin-jsdoc` providing a option to create an alias for the tag.
// e.g. Prefer `@foo` instead of `@param`.
// So, I decided to provide a generic text-based struct and let the user handle it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    raw_body: &'a str,
    pub kind: &'a str,
}

impl<'a> JSDocTag<'a> {
    /// kind: Does not contain the `@` prefix
    /// raw_body: The body part of the tag, after the `@kind {HERE_MAY_BE_MULTILINE...}`
    pub fn new(kind: &'a str, raw_body: &'a str) -> JSDocTag<'a> {
        debug_assert!(!kind.starts_with('@'));
        Self { raw_body, kind }
    }

    /// Use for various simple tags like `@access`, `@deprecated`, ...etc.
    /// comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind comment
    /// @kind
    /// ```
    pub fn comment(&self) -> String {
        utils::trim_multiline_comment(self.raw_body)
    }

    /// Use for `@type`, `@satisfies`, ...etc.
    ///
    /// Variants:
    /// ```
    /// @kind {type}
    /// @kind
    /// ```
    pub fn r#type(&self) -> Option<&str> {
        utils::find_type_range(self.raw_body).map(|(start, end)| &self.raw_body[start..end])
    }

    /// Use for `@yields`, `@returns`, ...etc.
    /// comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind {type} comment
    /// @kind {type}
    /// @kind comment
    /// @kind
    /// ```
    pub fn type_comment(&self) -> (Option<&str>, String) {
        let type_part_range = utils::find_type_range(self.raw_body);
        // {type} comment
        // {type}
        if let Some((start, end)) = type_part_range {
            (
                Some(&self.raw_body[start..end]),
                // +1 for `}`
                utils::trim_multiline_comment(&self.raw_body[end + 1..]),
            )
        }
        // comment
        // (empty)
        else {
            (None, utils::trim_multiline_comment(self.raw_body))
        }
    }

    /// Use for `@param`, `@property`, `@typedef`, ...etc.
    /// comment can be multiline.
    ///
    /// Variants:
    /// ```
    /// @kind {type} name comment
    /// @kind {type} name
    /// @kind {type}
    /// @kind name comment
    /// @kind name
    /// @kind
    /// ```
    pub fn type_name_comment(&self) -> (Option<&str>, Option<&str>, String) {
        let type_part_range = utils::find_type_range(self.raw_body);
        if let Some((t_start, t_end)) = type_part_range {
            let type_part = &self.raw_body[t_start..t_end];
            // +1 for `}`
            let name_comment_part = &self.raw_body[t_end + 1..];
            let name_part_range = utils::find_name_range(name_comment_part);

            // {type} name comment
            // {type} name
            if let Some((n_start, n_end)) = name_part_range {
                (
                    Some(type_part),
                    Some(&name_comment_part[n_start..n_end]),
                    utils::trim_multiline_comment(&name_comment_part[n_end..]),
                )
            }
            // {type}
            else {
                (Some(type_part), None, String::new())
            }
        } else {
            let name_part_range = utils::find_name_range(self.raw_body);
            // name comment
            // name
            if let Some((n_start, n_end)) = name_part_range {
                (
                    None,
                    Some(&self.raw_body[n_start..n_end]),
                    utils::trim_multiline_comment(&self.raw_body[n_end..]),
                )
            }
            // (empty)
            else {
                (None, None, utils::trim_multiline_comment(self.raw_body))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::JSDocTag;

    #[test]
    fn parses_comment() {
        assert_eq!(JSDocTag::new("a", "").comment(), "");
        assert_eq!(JSDocTag::new("a", "c1").comment(), "c1");
        assert_eq!(JSDocTag::new("a", " c2 \n z ").comment(), "c2\nz");
        assert_eq!(JSDocTag::new("a", "* c3\n *  \n z \n\n").comment(), "c3\nz");
        assert_eq!(
            JSDocTag::new("a", "comment4 and {@inline tag}!").comment(),
            "comment4 and {@inline tag}!"
        );
    }

    #[test]
    fn parses_type() {
        assert_eq!(JSDocTag::new("t", "{t1}").r#type(), Some("t1"));
        assert_eq!(JSDocTag::new("t", "{t2} foo").r#type(), Some("t2"));
        assert_eq!(JSDocTag::new("t", " {t3 } ").r#type(), Some("t3 "));
        assert_eq!(JSDocTag::new("t", "  ").r#type(), None);
        assert_eq!(JSDocTag::new("t", "t4").r#type(), None);
        assert_eq!(JSDocTag::new("t", "{t5 ").r#type(), None);
        assert_eq!(JSDocTag::new("t", "{t6}\nx").r#type(), Some("t6"));
    }

    #[test]
    fn parses_type_comment() {
        assert_eq!(JSDocTag::new("r", "{t1} c1").type_comment(), (Some("t1"), "c1".to_string()));
        assert_eq!(JSDocTag::new("r", "{t2}").type_comment(), (Some("t2"), String::new()));
        assert_eq!(JSDocTag::new("r", "c3").type_comment(), (None, "c3".to_string()));
        assert_eq!(JSDocTag::new("r", "c4 foo").type_comment(), (None, "c4 foo".to_string()));
        assert_eq!(JSDocTag::new("r", "").type_comment(), (None, String::new()));
        assert_eq!(
            JSDocTag::new("r", "{t5}\nc5\n...").type_comment(),
            (Some("t5"), "c5\n...".to_string())
        );
        assert_eq!(
            JSDocTag::new("r", "{t6} - c6").type_comment(),
            (Some("t6"), "- c6".to_string())
        );
        assert_eq!(
            JSDocTag::new("r", "{{ 型: t7 }} : c7").type_comment(),
            (Some("{ 型: t7 }"), ": c7".to_string())
        );
    }

    #[test]
    fn parses_type_name_comment() {
        assert_eq!(
            JSDocTag::new("p", "{t1} n1 c1").type_name_comment(),
            (Some("t1"), Some("n1"), "c1".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "{t2} n2").type_name_comment(),
            (Some("t2"), Some("n2"), String::new())
        );
        assert_eq!(
            JSDocTag::new("p", "n3 c3").type_name_comment(),
            (None, Some("n3"), "c3".to_string())
        );
        assert_eq!(JSDocTag::new("p", "").type_name_comment(), (None, None, String::new()));
        assert_eq!(JSDocTag::new("p", "\n\n").type_name_comment(), (None, None, String::new()));
        assert_eq!(
            JSDocTag::new("p", "{t4} n4 c4\n...").type_name_comment(),
            (Some("t4"), Some("n4"), "c4\n...".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "{t5} n5 - c5").type_name_comment(),
            (Some("t5"), Some("n5"), "- c5".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "{t6}\nn6\nc6").type_name_comment(),
            (Some("t6"), Some("n6"), "c6".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "{t7}\nn7\nc\n7").type_name_comment(),
            (Some("t7"), Some("n7"), "c\n7".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "{t8}").type_name_comment(),
            (Some("t8"), None, String::new())
        );
    }
}

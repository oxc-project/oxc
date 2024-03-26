use super::utils;

// Initially, I attempted to parse into specific structures such as:
// - `@param {type} name comment`: `JSDocParameterTag { type, name, comment }`
// - `@returns {type} comment`: `JSDocReturnsTag { type, comment }`
// - `@whatever comment`: `JSDocUnknownTag { comment }`
// - etc...
//
// However, I discovered that some use cases, like `eslint-plugin-jsdoc`, provide an option to create an alias for the tag kind.
// .e.g. Preferring `@foo` instead of `@param`
//
// This means that:
// - We cannot parse a tag exactly as it was written
// - We cannot assume that `@param` will always map to `JSDocParameterTag`
//
// Therefore, I decided to provide a generic structure with helper methods to parse the tag according to the needs.
//
// I also considered providing an API with methods like `as_param() -> JSDocParameterTag` or `as_return() -> JSDocReturnTag`, etc.
//
// However:
// - There are many kinds of tags, but most of them have a similar structure
// - JSDoc is not a strict format; it's just a comment
// - Users can invent their own tags like `@whatever {type}` and may want to parse its type
//
// As a result, I ended up providing helper methods that are fit for purpose.

/// General struct for JSDoc tag.
///
/// `kind` can be any string like `param`, `type`, `whatever`, ...etc.
/// `raw_body` is kept as is, you can use helper methods according to your needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    raw_body: &'a str,
    pub kind: &'a str,
}

impl<'a> JSDocTag<'a> {
    /// kind: Does not contain the `@` prefix
    /// raw_body: The body part of the tag, after the `@kind{HERE_MAY_BE_MULTILINE...}`
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
        utils::trim_comment(self.raw_body)
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
        let (type_part, comment_part) = match utils::find_type_range(self.raw_body) {
            Some((t_start, t_end)) => {
                // +1 for `}`, +1 for whitespace
                let c_start = self.raw_body.len().min(t_end + 2);
                (Some(&self.raw_body[t_start..t_end]), &self.raw_body[c_start..])
            }
            None => (None, self.raw_body),
        };

        (type_part, utils::trim_comment(comment_part))
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
        let (type_part, name_comment_part) = match utils::find_type_range(self.raw_body) {
            Some((t_start, t_end)) => {
                // +1 for `}`, +1 for whitespace
                let c_start = self.raw_body.len().min(t_end + 2);
                (Some(&self.raw_body[t_start..t_end]), &self.raw_body[c_start..])
            }
            None => (None, self.raw_body),
        };

        let (name_part, comment_part) = match utils::find_token_range(name_comment_part) {
            Some((n_start, n_end)) => {
                // +1 for whitespace
                let c_start = name_comment_part.len().min(n_end + 1);
                (Some(&name_comment_part[n_start..n_end]), &name_comment_part[c_start..])
            }
            None => (None, ""),
        };

        (type_part, name_part, utils::trim_comment(comment_part))
    }
}

#[cfg(test)]
mod test {
    use super::JSDocTag;

    #[test]
    fn parses_comment() {
        assert_eq!(JSDocTag::new("a", "").comment(), "");
        assert_eq!(JSDocTag::new("a", " c1").comment(), "c1");
        assert_eq!(JSDocTag::new("a", "\nc2 \n z ").comment(), "c2\nz");
        assert_eq!(JSDocTag::new("a", "\n* c3\n *  \n z \n\n").comment(), "c3\nz");
        assert_eq!(
            JSDocTag::new("a", " comment4 and {@inline tag}!").comment(),
            "comment4 and {@inline tag}!"
        );
    }

    #[test]
    fn parses_type() {
        assert_eq!(JSDocTag::new("t", " {t1}").r#type(), Some("t1"));
        assert_eq!(JSDocTag::new("t", "\n{t2} foo").r#type(), Some("t2"));
        assert_eq!(JSDocTag::new("t", " {t3 } ").r#type(), Some("t3 "));
        assert_eq!(JSDocTag::new("t", "  ").r#type(), None);
        assert_eq!(JSDocTag::new("t", " t4").r#type(), None);
        assert_eq!(JSDocTag::new("t", " {t5 ").r#type(), None);
        assert_eq!(JSDocTag::new("t", " {t6}\nx").r#type(), Some("t6"));
    }

    #[test]
    fn parses_type_comment() {
        assert_eq!(JSDocTag::new("r", " {t1} c1").type_comment(), (Some("t1"), "c1".to_string()));
        assert_eq!(JSDocTag::new("r", "\n{t2}").type_comment(), (Some("t2"), String::new()));
        assert_eq!(JSDocTag::new("r", " c3").type_comment(), (None, "c3".to_string()));
        assert_eq!(JSDocTag::new("r", " c4 foo").type_comment(), (None, "c4 foo".to_string()));
        assert_eq!(JSDocTag::new("r", " ").type_comment(), (None, String::new()));
        assert_eq!(
            JSDocTag::new("r", "\n{t5}\nc5\n...").type_comment(),
            (Some("t5"), "c5\n...".to_string())
        );
        assert_eq!(
            JSDocTag::new("r", " {t6} - c6").type_comment(),
            (Some("t6"), "- c6".to_string())
        );
        assert_eq!(
            JSDocTag::new("r", " {{ 型: t7 }} : c7").type_comment(),
            (Some("{ 型: t7 }"), ": c7".to_string())
        );
    }

    #[test]
    fn parses_type_name_comment() {
        assert_eq!(
            JSDocTag::new("p", " {t1} n1 c1").type_name_comment(),
            (Some("t1"), Some("n1"), "c1".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", " {t2} n2").type_name_comment(),
            (Some("t2"), Some("n2"), String::new())
        );
        assert_eq!(
            JSDocTag::new("p", " n3 c3").type_name_comment(),
            (None, Some("n3"), "c3".to_string())
        );
        assert_eq!(JSDocTag::new("p", "").type_name_comment(), (None, None, String::new()));
        assert_eq!(JSDocTag::new("p", "\n\n").type_name_comment(), (None, None, String::new()));
        assert_eq!(
            JSDocTag::new("p", " {t4} n4 c4\n...").type_name_comment(),
            (Some("t4"), Some("n4"), "c4\n...".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", " {t5} n5 - c5").type_name_comment(),
            (Some("t5"), Some("n5"), "- c5".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "\n{t6}\nn6\nc6").type_name_comment(),
            (Some("t6"), Some("n6"), "c6".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", "\n\n{t7}\nn7\nc\n7").type_name_comment(),
            (Some("t7"), Some("n7"), "c\n7".to_string())
        );
        assert_eq!(
            JSDocTag::new("p", " {t8}").type_name_comment(),
            (Some("t8"), None, String::new())
        );
    }
}

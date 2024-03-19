use super::utils;

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

    /// Use for simple tags like `@access`, `@deprecated`, ...etc.
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
        utils::extract_type_range(self.raw_body).map(|(start, end)| &self.raw_body[start..end])
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
        let type_part_range = utils::extract_type_range(self.raw_body);
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
        let type_part_range = utils::extract_type_range(self.raw_body);
        if let Some((t_start, t_end)) = type_part_range {
            let type_part = &self.raw_body[t_start..t_end];
            let name_comment_part = &self.raw_body[t_end + 1..];
            let name_part_range = utils::extract_name_range(name_comment_part);

            // {type} name comment
            // {type} name
            if let Some((n_start, n_end)) = name_part_range {
                (
                    Some(type_part),
                    Some(&name_comment_part[n_start..n_end]),
                    if n_end < name_comment_part.len() {
                        // +1 for ` ` or `\n`
                        utils::trim_multiline_comment(&name_comment_part[n_end + 1..])
                    } else {
                        String::new()
                    },
                )
            }
            // {type}
            else {
                (Some(type_part), Some(name_comment_part), String::new())
            }
        } else {
            let name_part_range = utils::extract_name_range(self.raw_body);
            // name comment
            // name
            if let Some((n_start, n_end)) = name_part_range {
                (
                    None,
                    Some(&self.raw_body[n_start..n_end]),
                    // +1 for ` ` or `\n`
                    utils::trim_multiline_comment(&self.raw_body[n_end + 1..]),
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
        assert_eq!(JSDocTag::new("foo1", "").comment(), "");
        assert_eq!(JSDocTag::new("foo2", "bar").comment(), "bar");
        assert_eq!(JSDocTag::new("foo3", " a \n z ").comment(), "a\nz");
        assert_eq!(JSDocTag::new("foo4", "* a\n *  \n z \n\n").comment(), "a\nz");
        assert_eq!(
            JSDocTag::new("foo5", "comment and {@inline tag}!").comment(),
            "comment and {@inline tag}!"
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
    }

    //         assert_eq!(
    //             parse_from_full_text("/** @param @noop */").1,
    //             vec![
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag { kind: JSDocTagKind::Unknown("noop"), comment: String::new() },
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param name */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param { name: "name", r#type: None }),
    //                 comment: String::new(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param {
    //                     name: "name",
    //                     r#type: Some(ParamType { value: "str" })
    //                 }),
    //                 comment: String::new(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param {
    //                     name: "name",
    //                     r#type: Some(ParamType { value: "str" })
    //                 }),
    //                 comment: "comment".to_string(),
    //             },]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */"),
    //             parse_from_full_text("/** @param {str} name - comment */"),
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @param {str} name comment */"),
    //             parse_from_full_text(
    //                 "/** @param {str} name
    //     comment */"
    //             ),
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/** @param {str} name
    //     comment */"
    //             ),
    //             parse_from_full_text(
    //                 "/**
    //                       * @param {str} name
    //                       * comment
    //                       */"
    //             ),
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //                     /**
    //                      * @param {boolean} a
    //                      * @param {string b
    //                      * @param {string} c comment
    //                      * @param {Num} d - comment2
    //                      */
    //             "
    //             )
    //             .1,
    //             vec![
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "a",
    //                         r#type: Some(ParamType { value: "boolean" })
    //                     }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "b",
    //                         r#type: Some(ParamType { value: "string" })
    //                     }),
    //                     comment: String::new(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "c",
    //                         r#type: Some(ParamType { value: "string" })
    //                     }),
    //                     comment: "comment".to_string(),
    //                 },
    //                 JSDocTag {
    //                     kind: JSDocTagKind::Parameter(Param {
    //                         name: "d",
    //                         r#type: Some(ParamType { value: "Num" })
    //                     }),
    //                     comment: "comment2".to_string(),
    //                 },
    //             ]
    //         );
    //     }
}

use super::utils;

//
// Structs
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    raw_body: &'a str,
    pub kind: &'a str,
}

impl<'a> JSDocTag<'a> {
    /// kind: Does not contain the `@` prefix
    /// raw_body: The body part of the tag, after the `@kind {HERE_MAY_BE_MULTILINE...}`
    pub fn new(kind: &'a str, raw_body: &'a str) -> JSDocTag<'a> {
        Self { raw_body, kind }
    }

    pub fn comment(&self) -> String {
        utils::trim_multiline_comment(self.raw_body)
    }

    // For `@type {type}`, `@satisfies {type}`, ...etc
    // It may be `@kind`
    pub fn r#type(&self) -> Option<&str> {
        let parts = self.body_splitn(1);
        parts.first().map(|may_type| utils::extract_type(may_type))?
    }

    // For `@yields {type} comment`, `@returns {type} comment`, ...etc
    // It may be `@kind {type}` or `@kind comment`
    // even or `@kind`
    // pub fn type_comment(&self) -> (Option<&str>, Option<String>) {}

    // For `@param {type} name comment`, `@property {type} name comment`, `@typedef {type} name comment`, ...etc
    // It may be `@kind {type} name` or `@kind name comment`,
    // even or `@kind {type}` or `@kind name`
    // even or `@kind`
    // pub fn type_name_comment(&self) -> (Option<&str>, Option<&str>, Option<String>) {}

    pub fn body_splitn(&self, max_parts: usize) -> Vec<&str> {
        debug_assert!(1 <= max_parts);
        debug_assert!(max_parts <= 3);

        let mut breakpoints = vec![];
        let mut in_braces = false;
        // Use indices for string slices
        let mut chars = self.raw_body.char_indices().peekable();

        // Skip leading spaces
        while let Some((_, ch)) = chars.peek() {
            if !(*ch == ' ' || *ch == '\n') {
                break;
            }
            chars.next();
        }

        'outer: while let Some((_, ch)) = chars.peek() {
            // To get 1 part, we need 0 breakpoints
            // To get 3 parts, we need 2 breakpoints
            if max_parts - 1 == breakpoints.len() {
                break;
            }

            match ch {
                '{' => in_braces = true,
                '}' => in_braces = false,
                ' ' | '\n' if !in_braces => {
                    for (idx, ch) in chars.by_ref() {
                        if ch != ' ' {
                            breakpoints.push(idx);
                            continue 'outer;
                        }
                    }
                }
                _ => {}
            }

            chars.next();
        }

        println!("Breakpoints: {breakpoints:?}");

        match max_parts {
            3 => {
                let idx1 = breakpoints[0];
                let idx2 = breakpoints[1];
                vec![&self.raw_body[..idx1], &self.raw_body[idx1..idx2], &self.raw_body[idx2..]]
            }
            2 => {
                let idx = breakpoints[0];
                vec![&self.raw_body[..idx], &self.raw_body[idx..]]
            }
            _ => {
                vec![self.raw_body]
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

    // #[test]
    // fn parses_parameter_tag() {
    //     assert_eq!(JSDocTag::new("param", "name1").as_param(), (None, Some("name1"), None));
    //     assert_eq!(
    //         JSDocTag::new("arg", "{type2} name2").as_param(),
    //         (Some("type2"), Some("name2"), None)
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", " {type3 }  name3 ").as_param(),
    //         (Some("type3"), Some("name3"), None)
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", "{{ x: 1 }} name4").as_param(),
    //         (Some("{ x: 1 }"), Some("name4"), None)
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", "{type5} name5 comment5").as_param(),
    //         (Some("type5"), Some("name5"), Some("comment5".to_string()))
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", "{type6} 変数6 あいうえ\nお6").as_param(),
    //         (Some("type6"), Some("変数6"), Some("あいうえ\nお6".to_string()))
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", "{type7}\nname7").as_param(),
    //         (Some("type7"), Some("name7"), None)
    //     );
    //     assert_eq!(
    //         JSDocTag::new("arg", "{type8}\nname8\ncomment8").as_param(),
    //         (Some("type8"), Some("name8"), Some("comment8".to_string()))
    //     );
    //     assert_eq!(JSDocTag::new("arg", "\nname9").as_param(), (None, Some("name9"), None));
    //     assert_eq!(
    //         JSDocTag::new("arg", "name10\ncom\nment10").as_param(),
    //         (None, Some("name10"), Some("com\nment10".to_string()))
    //     );
    //     assert_eq!(JSDocTag::new("arg", "{type11}").as_param(), (Some("type11"), None, None));

    //     // TODO: More tests!
    // }

    //         assert_eq!(
    //             parse_from_full_text("/** @param */").1,
    //             vec![JSDocTag {
    //                 kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
    //                 comment: String::new(),
    //             },]
    //         );
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

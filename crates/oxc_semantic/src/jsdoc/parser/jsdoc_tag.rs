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
    /// raw_body: The body part of the tag, after the `@kind {HERE...}`
    pub fn new(kind: &'a str, raw_body: &'a str) -> JSDocTag<'a> {
        Self { raw_body, kind }
    }

    pub fn comment(&self) -> String {
        utils::trim_multiline_comment(self.raw_body)
    }

    // Basic pattern:
    // ```
    // @param name1
    // @param {type} name2
    // @param {type} name3 comment
    // ```
    //
    // Advanced pattern:
    // ```
    // @param {type} name4 comment can go...
    // next line
    // @param
    // {type}
    // name5
    // comment...
    // ```
    pub fn as_param(&self) -> (Option<String>, Option<String>, Option<String>) {
        let mut breakpoints = vec![];

        let mut in_braces = false;
        for (i, ch) in self.raw_body.trim_start().char_indices() {
            match ch {
                '{' => in_braces = true,
                '}' => in_braces = false,
                ' ' | '\n' if !in_braces => {
                    breakpoints.push(i);
                }
                _ => {}
            }

            if breakpoints.len() == 2 {
                break;
            }
        }

        match breakpoints.len() {
            // name1
            0 => {
                let name = &self.raw_body[..].trim();
                (None, Some((*name).to_string()), None)
            }
            // {type} name2
            1 => {
                let r#type = &self.raw_body[..breakpoints[0]].trim();
                let r#type = &r#type[1..r#type.len() - 1];
                let name = &self.raw_body[breakpoints[0]..].trim();
                (Some(r#type.to_string()), Some((*name).to_string()), None)
            }
            // {type} name3 comment
            2 => {
                let r#type = &self.raw_body[..breakpoints[0]].trim();
                let r#type = &r#type[1..r#type.len() - 1];
                let name = &self.raw_body[breakpoints[0]..breakpoints[1]].trim();
                let comment = &self.raw_body[breakpoints[1]..];
                (
                    Some(r#type.to_string()),
                    Some((*name).to_string()),
                    Some(utils::trim_multiline_comment(comment)),
                )
            }
            // Unreachable!
            _ => (None, None, None),
        }
    }

    // pub fn body_as_returns(&self) {}
}

#[cfg(test)]
mod test {
    use super::JSDocTag;

    #[test]
    fn parses_comment() {
        assert_eq!(JSDocTag::new("foo1", "").comment(), "");
        assert_eq!(JSDocTag::new("foo2", "bar").comment(), "bar");
        assert_eq!(JSDocTag::new("foo3", " ba \n z ").comment(), "ba\nz");
        assert_eq!(JSDocTag::new("foo4", "* ba\n *  \n z \n\n").comment(), "ba\nz");
        assert_eq!(
            JSDocTag::new("foo5", "comment and {@inline tag}!").comment(),
            "comment and {@inline tag}!"
        );
    }

    #[test]
    fn parses_parameter_tag() {
        assert_eq!(
            JSDocTag::new("param", "name").as_param(),
            (None, Some("name".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{type} name").as_param(),
            (Some("type".to_string()), Some("name".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{{ x: 1 }} name").as_param(),
            (Some("{ x: 1 }".to_string()), Some("name".to_string()), None)
        );
        assert_eq!(
            JSDocTag::new("arg", "{type} name comment").as_param(),
            (Some("type".to_string()), Some("name".to_string()), Some("comment".to_string()))
        );

        // TODO: More tests!
    }

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

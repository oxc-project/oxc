use super::jsdoc_tag::{JSDocTag, JSDocTagKind};
use super::jsdoc_tag::{Param, ParamType};
use super::utils;

#[derive(Debug)]
pub struct JSDocParser<'a> {
    source_text: &'a str,
    current: usize,
}

// Refs: `parseJSDocCommentWorker()` and `doJSDocScan()` from TypeScript
// https://github.com/microsoft/TypeScript/blob/df8d755c1d76eaf0a8f1c1046a46061b53315718/src/compiler/parser.ts#L8814
impl<'a> JSDocParser<'a> {
    /// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
    pub fn new(source_text: &'a str) -> Self {
        // Outer spaces can be trimmed
        Self { source_text: source_text.trim(), current: 0 }
    }

    pub fn parse(mut self) -> (String, Vec<JSDocTag<'a>>) {
        let comment = self.parse_comment();
        let tags = self.parse_tags();

        (comment, tags)
    }

    // JSDoc comment starts with description comment until the first `@` appears
    fn parse_comment(&mut self) -> String {
        // TODO: Should ignore inside of inline tags like `{@link}`?
        let comment = self.take_until(|c| c == '@');
        utils::trim_multiline_comment(comment)
    }

    fn parse_tags(&mut self) -> Vec<JSDocTag<'a>> {
        let mut tags = vec![];

        // Let's start with the first `@`
        while let Some(c) = self.source_text[self.current..].chars().next() {
            match c {
                '@' => {
                    self.current += c.len_utf8();
                    tags.push(self.parse_tag());
                }
                _ => {
                    self.current += c.len_utf8();
                }
            }
        }

        tags
    }

    fn parse_tag(&mut self) -> JSDocTag<'a> {
        let tag_name = self.take_until(|c| c == ' ' || c == '\n' || c == '@');
        match tag_name {
            // TODO: Add more tags
            "access" => self.parse_simple_tag(JSDocTagKind::Access),
            "package" => self.parse_simple_tag(JSDocTagKind::Package),
            "private" => self.parse_simple_tag(JSDocTagKind::Private),
            "protected" => self.parse_simple_tag(JSDocTagKind::Protected),
            "public" => self.parse_simple_tag(JSDocTagKind::Public),
            "arg" | "argument" | "param" => self.parse_parameter_tag(),
            "deprecated" => self.parse_simple_tag(JSDocTagKind::Deprecated),
            _ => self.parse_simple_tag(JSDocTagKind::Unknown(tag_name)),
        }
    }

    // @tag_name [<some text>]
    fn parse_simple_tag(&mut self, kind: JSDocTagKind<'a>) -> JSDocTag<'a> {
        let comment = self.take_until(|c| c == '@');
        let comment = utils::trim_multiline_comment(comment);
        JSDocTag { kind, comment }
    }

    // @param name
    // @param {type} name
    // @param {type} name comment
    // @param {type} name - comment
    fn parse_parameter_tag(&mut self) -> JSDocTag<'a> {
        self.skip_whitespace();

        let mut r#type = None;
        if self.at('{') {
            // If we hit a space, then treat it as the end of the type annotation.
            let type_annotation = self.take_until(|c| c == '}' || c == ' ' || c == '@');
            r#type = Some(ParamType { value: type_annotation });
            if self.at('}') {
                self.skip_whitespace();
            }
            self.skip_whitespace();
        }

        let name = self.take_until(|c| c == ' ' || c == '\n' || c == '@');
        let param = Param { name, r#type };

        self.skip_whitespace();

        // JSDoc.app ignores `-` char between name and comment, but TS doesn't
        // Some people use `:` as separator
        if self.at('-') || self.at(':') {
            self.skip_whitespace();
        }

        let comment = self.take_until(|c| c == '@');
        let comment = utils::trim_multiline_comment(comment);
        JSDocTag { kind: JSDocTagKind::Parameter(param), comment }
    }

    //
    // Parser utils
    //
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.source_text[self.current..].chars().next() {
            if c != ' ' {
                break;
            }
            self.current += c.len_utf8();
        }
    }

    fn advance(&mut self) {
        if let Some(c) = self.source_text[self.current..].chars().next() {
            self.current += c.len_utf8();
        }
    }

    fn at(&mut self, c: char) -> bool {
        if let Some(ch) = self.source_text[self.current..].chars().next() {
            if ch == c {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn take_until(&mut self, predicate: fn(char) -> bool) -> &'a str {
        let start = self.current;
        while let Some(c) = self.source_text[self.current..].chars().next() {
            if predicate(c) {
                break;
            }
            self.current += c.len_utf8();
        }
        &self.source_text[start..self.current]
    }
}

#[cfg(test)]
mod test {
    use super::JSDocParser;
    use super::{JSDocTag, JSDocTagKind};
    use super::{Param, ParamType};

    fn parse_from_full_text(full_text: &str) -> (String, Vec<JSDocTag>) {
        // Outside of markers can be trimmed
        let source_text = full_text.trim().trim_start_matches("/**").trim_end_matches("*/");
        JSDocParser::new(source_text).parse()
    }

    #[test]
    fn parses_jsdoc_comment() {
        assert_eq!(JSDocParser::new("hello source").parse().0, "hello source");
        assert_eq!(parse_from_full_text("/** hello full */").0, "hello full");

        assert_eq!(JSDocParser::new(" <- trim -> ").parse().0, "<- trim ->");
        assert_eq!(
            parse_from_full_text(
                "
        /**
         * <- omit this, keep this -> *
         */
        "
            )
            .0,
            "<- omit this, keep this -> *"
        );

        assert_eq!(
            parse_from_full_text(
                "/**
this is
comment
@x
*/"
            )
            .0,
            "this is\ncomment"
        );
        assert_eq!(
            parse_from_full_text(
                "/**
　　　　　　　　　* 日本語とか
　　　　　　　　　* multibyte文字はどう？
                  */"
            )
            .0,
            "日本語とか\nmultibyte文字はどう？"
        );
    }

    #[test]
    fn parses_single_line_1_jsdoc() {
        assert_eq!(
            JSDocParser::new("@deprecated").parse().1,
            parse_from_full_text("/** @deprecated */").1,
        );
        assert_eq!(
            JSDocParser::new("@deprecated").parse().1,
            vec![JSDocTag { kind: JSDocTagKind::Deprecated, comment: String::new() }]
        );

        assert_eq!(
            parse_from_full_text("/**@foo since 2024 */").1,
            vec![JSDocTag {
                kind: JSDocTagKind::Unknown("foo"),
                comment: "since 2024".to_string()
            }]
        );
        assert_eq!(
            parse_from_full_text("/**@*/").1,
            vec![JSDocTag { kind: JSDocTagKind::Unknown(""), comment: String::new() }]
        );
    }

    #[test]
    fn parses_single_line_n_jsdocs() {
        assert_eq!(
            parse_from_full_text("/** @foo @bar */").1,
            vec![
                JSDocTag { kind: JSDocTagKind::Unknown("foo"), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown("bar"), comment: String::new() }
            ]
        );
        assert_eq!(
            parse_from_full_text("/** @a @@ @d */").1,
            vec![
                JSDocTag { kind: JSDocTagKind::Unknown("a"), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown(""), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown(""), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown("d"), comment: String::new() }
            ]
        );
    }

    #[test]
    fn parses_multiline_1_jsdoc() {
        assert_eq!(
            parse_from_full_text(
                "/** @yo
*/"
            )
            .1,
            vec![JSDocTag { kind: JSDocTagKind::Unknown("yo"), comment: String::new() }]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                      * @foo
                      */"
            )
            .1,
            vec![JSDocTag { kind: JSDocTagKind::Unknown("foo"), comment: String::new() }]
        );
        assert_eq!(
            parse_from_full_text(
                "
    /**
     * @x with asterisk
     */
            "
            )
            .1,
            vec![JSDocTag {
                kind: JSDocTagKind::Unknown("x"),
                comment: "with asterisk".to_string()
            }]
        );
        assert_eq!(
            parse_from_full_text(
                "
    /**
    @y without
asterisk
     */
            "
            )
            .1,
            vec![JSDocTag {
                kind: JSDocTagKind::Unknown("y"),
                comment: "without\nasterisk".to_string()
            }]
        );
    }

    #[test]
    fn parses_multiline_n_jsdocs() {
        assert_eq!(
            parse_from_full_text(
                "
    /**
       @foo      @bar
    * @baz
     */
            "
            )
            .1,
            vec![
                JSDocTag { kind: JSDocTagKind::Unknown("foo"), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown("bar"), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown("baz"), comment: String::new() },
            ]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                      * @one
                  *
                  * ...
              *
                      * @two
                  */"
            )
            .1,
            vec![
                JSDocTag { kind: JSDocTagKind::Unknown("one"), comment: "...".to_string() },
                JSDocTag { kind: JSDocTagKind::Unknown("two"), comment: String::new() },
            ]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                  * ...
                  * @hey you!
                  *   Are you OK?
                  * @yes I'm fine
                  */"
            )
            .1,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Unknown("hey"),
                    comment: "you!\nAre you OK?".to_string()
                },
                JSDocTag { kind: JSDocTagKind::Unknown("yes"), comment: "I'm fine".to_string() },
            ]
        );
    }

    #[test]
    fn parses_parameter_tag() {
        assert_eq!(
            parse_from_full_text("/** @param */").1,
            vec![JSDocTag {
                kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
                comment: String::new(),
            },]
        );
        assert_eq!(
            parse_from_full_text("/** @param @noop */").1,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param { name: "", r#type: None }),
                    comment: String::new(),
                },
                JSDocTag { kind: JSDocTagKind::Unknown("noop"), comment: String::new() },
            ]
        );
        assert_eq!(
            parse_from_full_text("/** @param name */").1,
            vec![JSDocTag {
                kind: JSDocTagKind::Parameter(Param { name: "name", r#type: None }),
                comment: String::new(),
            },]
        );
        assert_eq!(
            parse_from_full_text("/** @param {str} name */").1,
            vec![JSDocTag {
                kind: JSDocTagKind::Parameter(Param {
                    name: "name",
                    r#type: Some(ParamType { value: "str" })
                }),
                comment: String::new(),
            },]
        );
        assert_eq!(
            parse_from_full_text("/** @param {str} name comment */").1,
            vec![JSDocTag {
                kind: JSDocTagKind::Parameter(Param {
                    name: "name",
                    r#type: Some(ParamType { value: "str" })
                }),
                comment: "comment".to_string(),
            },]
        );
        assert_eq!(
            parse_from_full_text("/** @param {str} name comment */"),
            parse_from_full_text("/** @param {str} name - comment */"),
        );
        assert_eq!(
            parse_from_full_text("/** @param {str} name comment */"),
            parse_from_full_text(
                "/** @param {str} name
comment */"
            ),
        );
        assert_eq!(
            parse_from_full_text(
                "/** @param {str} name
comment */"
            ),
            parse_from_full_text(
                "/**
                  * @param {str} name
                  * comment
                  */"
            ),
        );

        assert_eq!(
            parse_from_full_text(
                "
                /**
                 * @param {boolean} a
                 * @param {string b
                 * @param {string} c comment
                 * @param {Num} d - comment2
                 */
        "
            )
            .1,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "a",
                        r#type: Some(ParamType { value: "boolean" })
                    }),
                    comment: String::new(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "b",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    comment: String::new(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "c",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    comment: "comment".to_string(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "d",
                        r#type: Some(ParamType { value: "Num" })
                    }),
                    comment: "comment2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parses_practical_with_multibyte() {
        let jsdoc = parse_from_full_text(
            "/**
              * flat tree data on expanded state
              *
              * @export
              * @template T
              * @param {*} data : table data
              * @param {string} childrenColumnName : 指定树形结构的列名
              * @param {Set<Key>} expandedKeys : 展开的行对应的keys
              * @param {GetRowKey<T>} getRowKey  : 获取当前rowKey的方法
              * @returns flattened data
              */",
        );
        assert_eq!(jsdoc.0, "flat tree data on expanded state");
        assert_eq!(
            jsdoc.1,
            vec![
                JSDocTag { kind: JSDocTagKind::Unknown("export"), comment: String::new() },
                JSDocTag { kind: JSDocTagKind::Unknown("template"), comment: "T".to_string() },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "data",
                        r#type: Some(ParamType { value: "*" })
                    }),
                    comment: "table data".to_string(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "childrenColumnName",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    comment: "指定树形结构的列名".to_string(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "expandedKeys",
                        r#type: Some(ParamType { value: "Set<Key>" })
                    }),
                    comment: "展开的行对应的keys".to_string(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "getRowKey",
                        r#type: Some(ParamType { value: "GetRowKey<T>" })
                    }),
                    comment: "获取当前rowKey的方法".to_string(),
                },
                JSDocTag {
                    kind: JSDocTagKind::Unknown("returns"),
                    comment: "flattened data".to_string(),
                },
            ]
        );
    }
}

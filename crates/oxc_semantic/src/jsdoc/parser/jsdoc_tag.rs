use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamTypeKind {
    Any,
    Repeated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamType<'a> {
    value: &'a str,
}

impl<'a> ParamType<'a> {
    #[allow(unused)]
    pub fn kind(&self) -> Option<ParamTypeKind> {
        ParamTypeKind::from_str(self.value).map(Option::Some).unwrap_or_default()
    }
}

impl FromStr for ParamTypeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: This might be inaccurate if the type is listed as {....string} or some variant
        if s.len() > 3 && &s[0..3] == "..." {
            return Ok(Self::Repeated);
        }

        if s == "*" {
            return Ok(Self::Any);
        }

        Err(())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param<'a> {
    name: &'a str,
    r#type: Option<ParamType<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JSDocTagKind<'a> {
    Deprecated,
    Parameter(Param<'a>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag<'a> {
    pub kind: JSDocTagKind<'a>,
    pub description: &'a str,
}

impl<'a> JSDocTag<'a> {
    pub fn is_deprecated(&self) -> bool {
        matches!(self.kind, JSDocTagKind::Deprecated)
    }
}

#[derive(Debug)]
pub struct JSDocTagParser<'a> {
    source_text: &'a str,
    current: usize,
}

// Refs: `parseJSDocCommentWorker()` and `doJSDocScan()` from TypeScript
// https://github.com/microsoft/TypeScript/blob/df8d755c1d76eaf0a8f1c1046a46061b53315718/src/compiler/parser.ts#L8814

// TODO: Parse multiline descriptions
// ```
// /**
//  * @param {string} a - descripti~
//  *   ~on and more
//  */
// ```
// This should be "descripti~\n~on and more"
// TODO: `@` can only appear at the start of a line
// TODO: tag_name() -> &str for Unknown

impl<'a> JSDocTagParser<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text, current: 0 }
    }

    pub fn parse(mut self) -> Vec<JSDocTag<'a>> {
        self.parse_comment(self.source_text)
    }

    fn advance(&mut self) {
        if self.current < self.source_text.len() {
            self.current += 1;
        }
    }

    fn at(&mut self, c: char) -> bool {
        let Some(ch) = self.source_text.chars().nth(self.current) else { return false };
        if ch == c {
            self.advance();
            true
        } else {
            false
        }
    }

    fn take_until(&mut self, s: &'a str, predicate: fn(char) -> bool) -> &'a str {
        let start = self.current;
        while let Some(c) = s.chars().nth(self.current) {
            if predicate(c) {
                break;
            }
            self.current += 1;
        }
        &s[start..self.current]
    }

    fn skip_whitespace(&mut self, s: &'a str) {
        while let Some(c) = s.chars().nth(self.current) {
            if c != ' ' {
                break;
            }
            self.current += 1;
        }
    }

    fn parse_comment(&mut self, comment: &'a str) -> Vec<JSDocTag<'a>> {
        let mut tags = vec![];

        while let Some(c) = comment.chars().nth(self.current) {
            match c {
                '@' => {
                    self.current += 1;

                    let tag = self.parse_tag(comment);
                    self.current += tag.description.len();

                    tags.push(tag);
                }
                _ => {
                    self.current += 1;
                }
            }
        }

        tags
    }

    // See also https://jsdoc.app/#block-tags
    fn parse_tag(&mut self, comment: &'a str) -> JSDocTag<'a> {
        let tag = self.take_until(comment, |c| c == ' ' || c == '\n');
        match tag {
            "deprecated" => self.parse_deprecated_tag(comment),
            "arg" | "argument" | "param" => self.parse_parameter_tag(comment),
            _ => self.parse_unknown_tag(comment),
        }
    }

    // @deprecated [<some text>]
    // https://jsdoc.app/tags-deprecated
    fn parse_deprecated_tag(&mut self, comment: &'a str) -> JSDocTag<'a> {
        self.skip_whitespace(comment);

        let description = self.take_until(comment, |c| c == '\n' || c == '*');
        JSDocTag { kind: JSDocTagKind::Deprecated, description }
    }

    // @param name
    // @param {type} name
    // @param {type} name description
    // @param {type} name - description
    // https://jsdoc.app/tags-param
    fn parse_parameter_tag(&mut self, comment: &'a str) -> JSDocTag<'a> {
        self.skip_whitespace(comment);

        let mut r#type = None;

        if self.at('{') {
            // If we hit a space, then treat it as the end of the type annotation.
            let type_annotation = self.take_until(comment, |c| c == '}' || c == ' ');
            r#type = Some(ParamType { value: type_annotation });
            if self.at('}') {
                self.skip_whitespace(comment);
            }
            self.skip_whitespace(comment);
        }

        let name = self.take_until(comment, |c| c == ' ' || c == '\n');

        self.skip_whitespace(comment);
        if self.at('-') {
            self.skip_whitespace(comment);
        }

        let description = self.take_until(comment, |c| c == '\n' || c == '*');

        JSDocTag { kind: JSDocTagKind::Parameter(Param { name, r#type }), description }
    }

    // Valid tag but not supported yet, truly unknown tag
    fn parse_unknown_tag(&mut self, comment: &'a str) -> JSDocTag<'a> {
        self.skip_whitespace(comment);

        let description = self.take_until(comment, |c| c == '\n' || c == '*');
        JSDocTag { kind: JSDocTagKind::Unknown, description }
    }
}

#[cfg(test)]
mod test {
    use super::JSDocTagParser;
    use super::{JSDocTag, JSDocTagKind, Param, ParamType, ParamTypeKind};

    #[test]
    fn deduces_correct_param_kind() {
        let param = Param { name: "a", r#type: Some(ParamType { value: "string" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), None);

        let param = Param { name: "a", r#type: Some(ParamType { value: "...string" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), Some(ParamTypeKind::Repeated));

        let param = Param { name: "a", r#type: Some(ParamType { value: "*" }) };
        assert_eq!(param.r#type.and_then(|t| t.kind()), Some(ParamTypeKind::Any));
    }

    #[test]
    fn parses_single_line_jsdoc() {
        let source = "/** @deprecated */";

        let tags = JSDocTagParser::new(source).parse();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags, vec![JSDocTag { kind: JSDocTagKind::Deprecated, description: "" }]);
    }

    #[test]
    fn parses_multi_line_disjoint_jsdoc() {
        let source = r"/** @deprecated
        */
        ";

        let tags = JSDocTagParser::new(source).parse();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags, vec![JSDocTag { kind: JSDocTagKind::Deprecated, description: "" }]);
    }

    #[test]
    fn parses_multiline_jsdoc() {
        let source = r"/**
        * @param a
        * @deprecated
        */
       ";

        let tags = JSDocTagParser::new(source).parse();
        assert_eq!(tags.len(), 2);
        assert_eq!(
            tags,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param { name: "a", r#type: None }),
                    description: ""
                },
                JSDocTag { kind: JSDocTagKind::Deprecated, description: "" },
            ]
        );
    }

    #[test]
    fn parses_multiline_jsdoc_with_descriptions() {
        let source = r"/**
        * @param a
        * @deprecated since version 1.0
        */
       ";

        let tags = JSDocTagParser::new(source).parse();
        assert_eq!(tags.len(), 2);
        assert_eq!(
            tags,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param { name: "a", r#type: None }),
                    description: ""
                },
                JSDocTag { kind: JSDocTagKind::Deprecated, description: "since version 1.0" },
            ]
        );
    }

    #[test]
    fn parses_param_type_annotation() {
        let source = r"/**
        * @param {string} a
        * @param {string b
        * @param {string} c - description
        */
       ";

        let tags = JSDocTagParser::new(source).parse();
        assert_eq!(tags.len(), 3);
        assert_eq!(
            tags,
            vec![
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "a",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    description: ""
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "b",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    description: ""
                },
                JSDocTag {
                    kind: JSDocTagKind::Parameter(Param {
                        name: "c",
                        r#type: Some(ParamType { value: "string" })
                    }),
                    description: "description"
                },
            ]
        );
    }
}

use super::utils;

//
// Structs
//

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JSDocTag {
    pub kind: String,
    raw_body: String,
}

impl JSDocTag {
    pub fn new(kind: String, raw_body: String) -> JSDocTag {
        Self { kind, raw_body }
    }

    pub fn comment(&self) -> String {
        utils::trim_multiline_comment(&self.raw_body)
    }
}

#[cfg(test)]
mod test {
    //     #[test]
    //     fn parses_parameter_tag() {
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
    // comment */"
    //             ),
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/** @param {str} name
    // comment */"
    //             ),
    //             parse_from_full_text(
    //                 "/**
    //                   * @param {str} name
    //                   * comment
    //                   */"
    //             ),
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //                 /**
    //                  * @param {boolean} a
    //                  * @param {string b
    //                  * @param {string} c comment
    //                  * @param {Num} d - comment2
    //                  */
    //         "
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

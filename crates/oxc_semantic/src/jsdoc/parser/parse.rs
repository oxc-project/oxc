use super::jsdoc_parts::{JSDocCommentPart, JSDocTagKindPart};
use super::jsdoc_tag::JSDocTag;
use super::utils;
use oxc_span::Span;

/// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
/// span_start: Global positioned `Span` start for this JSDoc comment
pub fn parse_jsdoc(source_text: &str, jsdoc_span_start: u32) -> (JSDocCommentPart, Vec<JSDocTag>) {
    debug_assert!(!source_text.starts_with("/*"));
    debug_assert!(!source_text.ends_with("*/"));

    // JSDoc consists of comment and tags.
    // - Comment goes first, and tags(`@xxx`) follow
    // - Both can be optional
    // - Each tag is also separated by whitespace + `@`
    let mut comment = None;
    let mut tags = vec![];

    // So, find `@` to split comment and each tag.
    // But `@` can be found inside of `{}` (e.g. `{@see link}`), it should be distinguished.
    let mut in_braces = false;
    let mut comment_found = false;
    // Parser local offsets, not for global span
    let (mut start, mut end) = (0, 0);
    for ch in source_text.chars() {
        match ch {
            '{' => in_braces = true,
            '}' => in_braces = false,
            '@' if !in_braces => {
                let part = &source_text[start..end];
                let span = Span::new(
                    jsdoc_span_start + u32::try_from(start).unwrap_or_default(),
                    jsdoc_span_start + u32::try_from(end).unwrap_or_default(),
                );

                if comment_found {
                    tags.push(parse_jsdoc_tag(part, span));
                } else {
                    comment = Some(JSDocCommentPart::new(part, span));
                    comment_found = true;
                }

                // Prepare for the next draft
                start = end;
            }
            _ => {}
        }
        // Update the current draft
        end += ch.len_utf8();
    }

    // If `@` not found, flush the last draft
    if start != end {
        let part = &source_text[start..end];
        let span = Span::new(
            jsdoc_span_start + u32::try_from(start).unwrap_or_default(),
            jsdoc_span_start + u32::try_from(end).unwrap_or_default(),
        );

        if comment_found {
            tags.push(parse_jsdoc_tag(part, span));
        } else {
            comment = Some(JSDocCommentPart::new(part, span));
        }
    }

    (
        comment.unwrap_or(JSDocCommentPart::new("", Span::new(jsdoc_span_start, jsdoc_span_start))),
        tags,
    )
}

/// tag_content: Starts with `@`, may be mulitline
fn parse_jsdoc_tag(tag_content: &str, jsdoc_tag_span: Span) -> JSDocTag {
    debug_assert!(tag_content.starts_with('@'));
    // This surely exists, at least `@` itself
    let (k_start, k_end) = utils::find_token_range(tag_content).unwrap();

    let kind = JSDocTagKindPart::new(
        &tag_content[k_start..k_end],
        Span::new(
            jsdoc_tag_span.start + u32::try_from(k_start).unwrap_or_default(),
            jsdoc_tag_span.start + u32::try_from(k_end).unwrap_or_default(),
        ),
    );

    JSDocTag::new(
        kind,
        // Includes splitter whitespace to distinguish these cases:
        // ```
        // /**
        //  * @k * <- should not omit
        //  */
        //
        // /**
        //  * @k
        //  * <- should omit
        //  */
        // ```
        // If not included, both body_part will starts with `* <- ...`!
        //
        // It does not affect the output since it will be trimmed later.
        &tag_content[k_end..],
        Span::new(
            // +1 for whitespace, which is noted above
            jsdoc_tag_span.start + u32::try_from(k_end + 1).unwrap_or_default(),
            jsdoc_tag_span.end,
        ),
    )
}

#[cfg(test)]
mod test {
    // use super::parse_jsdoc_tag;
    use crate::JSDoc;
    use oxc_span::Span;

    fn parse_from_full_text(full_text: &str) -> JSDoc {
        // Outside of markers can be trimmed
        let source_text = full_text.trim().trim_start_matches("/**").trim_end_matches("*/");
        let jsdoc = JSDoc::new(source_text, Span::new(3, 3));
        jsdoc
    }

    #[test]
    fn parses_jsdoc_comment() {
        for (full_text, expect_comment) in
            [("/**hello*/", "hello"), ("/** hello full_text */", "hello full_text")]
        {
            let jsdoc = parse_from_full_text(full_text);
            let comment_part = jsdoc.comment();
            assert_eq!(comment_part.parsed(), expect_comment.to_string());
            println!("`{}`", comment_part.span.source_text(full_text));
        }
        //         assert_eq!(
        //             parse_from_full_text("/** hello full_text */"),
        //             ("hello full_text".to_string(), vec![])
        //         );
        //         assert_eq!(parse_from_full_text("/***/"), (String::new(), vec![]));
        //         assert_eq!(parse_from_full_text("/****/"), ("*".to_string(), vec![]));
        //         assert_eq!(parse_from_full_text("/*****/"), ("**".to_string(), vec![]));
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        //                   * * x
        //                   ** y
        //                   */"
        //             )
        //             .0,
        //             "* x\n* y"
        //         );

        //         assert_eq!(parse_from_full_text("/** <- trim -> */").0, "<- trim ->");
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "
        //         /**
        //          * <- omit this, keep this -> *
        //          */
        //         "
        //             )
        //             .0,
        //             "<- omit this, keep this -> *"
        //         );

        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        // this is
        // comment {@link link} ...
        // @x
        // */"
        //             )
        //             .0,
        //             "this is\ncomment {@link link} ..."
        //         );
        //         assert_eq!(
        //             parse_from_full_text(
        //                 "/**
        // 　　　　　　　　　* 日本語とか
        // 　　　　　　　　　* multibyte文字はどう⁉️
        //                   */"
        //             )
        //             .0,
        //             "日本語とか\nmultibyte文字はどう⁉️"
        //         );

        //         assert_eq!(
        //             parse_from_full_text("/**\nhello {@see inline} source {@a 2}\n*/").0,
        //             "hello {@see inline} source {@a 2}"
        //         );

        //         assert_eq!(parse_from_full_text("/** ハロー @comment だよ*/").0, "ハロー");
    }

    //     #[test]
    //     fn parses_jsdoc_tags() {
    //         assert_eq!(
    //             parse_from_full_text("/**@deprecated*/").1,
    //             vec![parse_jsdoc_tag("@deprecated")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/**@foo since 2024 */").1,
    //             vec![parse_jsdoc_tag("@foo since 2024 ")]
    //         );

    //         assert_eq!(
    //             parse_from_full_text("/** @foo @bar */").1,
    //             vec![parse_jsdoc_tag("@foo "), parse_jsdoc_tag("@bar ")]
    //         );

    //         assert_eq!(parse_from_full_text("/**@*/").1, vec![parse_jsdoc_tag("@")]);

    //         assert_eq!(
    //             parse_from_full_text("/** @aiue あいうえ @o お*/").1,
    //             vec![parse_jsdoc_tag("@aiue あいうえ "), parse_jsdoc_tag("@o お")],
    //         );
    //         assert_eq!(
    //             parse_from_full_text("/** @a @@ @d */").1,
    //             vec![
    //                 parse_jsdoc_tag("@a "),
    //                 parse_jsdoc_tag("@"),
    //                 parse_jsdoc_tag("@ "),
    //                 parse_jsdoc_tag("@d ")
    //             ],
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/** @yo
    //     */"
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@yo\n    ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                     *     @foo
    //                           */"
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@foo\n                          ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //         /**
    //          * @x with asterisk
    //          */
    //                 "
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@x with asterisk\n         ")]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //             /**
    //             @y without
    //         asterisk
    //              */
    //                     "
    //             )
    //             .1,
    //             vec![parse_jsdoc_tag("@y without\n        asterisk\n             ")]
    //         );

    //         assert_eq!(
    //             parse_from_full_text(
    //                 "
    //     /**
    //        @foo@bar
    //     * @baz
    //      */
    //             "
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag("@foo"),
    //                 parse_jsdoc_tag("@bar\n    * "),
    //                 parse_jsdoc_tag("@baz\n     ")
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                       * @one
    //                   *
    //                   * ...
    //               *
    //                       * @two */"
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag("@one\n                  *\n                  * ...\n              *\n                      * "),
    //                 parse_jsdoc_tag("@two ")
    //             ]
    //         );
    //         assert_eq!(
    //             parse_from_full_text(
    //                 "/**
    //                   * ...
    //                   * @hey you!
    //                   *   Are you OK?
    //                   * @yes I'm fine
    //                   */"
    //             )
    //             .1,
    //             vec![
    //                 parse_jsdoc_tag(
    //                     "@hey you!\n                  *   Are you OK?\n                  * "
    //                 ),
    //                 parse_jsdoc_tag("@yes I'm fine\n                  ")
    //             ]
    //         );
    //     }

    //     #[test]
    //     fn parses_practical() {
    //         let jsdoc = parse_from_full_text(
    //             "
    // /**
    //  * @typedef {Object} User - a User account
    //  * @property {string} displayName - the name used to show the user
    //  * @property {number} id - a unique id
    //  */
    // ",
    //         );
    //         let mut tags = jsdoc.1.iter();
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "typedef");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "property");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "property");

    //         let jsdoc = parse_from_full_text(
    //             "
    // /**
    //  * Adds two numbers together
    //  * @param {number} a The first number
    //  * @param {number} b The second number
    //  * @returns {number}
    //  */
    // ",
    //         );
    //         let mut tags = jsdoc.1.iter();
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "returns");
    //     }

    //     #[test]
    //     fn parses_practical_with_multibyte() {
    //         let jsdoc = parse_from_full_text(
    //             "/**
    //               * flat tree data on expanded state
    //               *
    //               * @export
    //               * @template T
    //               * @param {*} data : table data
    //               * @param {string} childrenColumnName : 指定树形结构的列名
    //               * @param {Set<Key>} expandedKeys : 展开的行对应的keys
    //               * @param {GetRowKey<T>} getRowKey  : 获取当前rowKey的方法
    //               * @returns flattened data
    //               */",
    //         );
    //         assert_eq!(jsdoc.0, "flat tree data on expanded state");
    //         let mut tags = jsdoc.1.iter();
    //         assert_eq!(tags.len(), 7);

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "export");
    //         assert_eq!(tag.comment(), "");

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "template");
    //         assert_eq!(tag.comment(), "T");

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(tag.type_name_comment(), (Some("*"), Some("data"), ": table data".to_string()));

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("string"), Some("childrenColumnName"), ": 指定树形结构的列名".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("Set<Key>"), Some("expandedKeys"), ": 展开的行对应的keys".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "param");
    //         assert_eq!(
    //             tag.type_name_comment(),
    //             (Some("GetRowKey<T>"), Some("getRowKey"), ": 获取当前rowKey的方法".to_string())
    //         );

    //         let tag = tags.next().unwrap();
    //         assert_eq!(tag.kind, "returns");
    //         assert_eq!(tag.type_comment(), (None, "flattened data".to_string()));
    //     }
}

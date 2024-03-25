use super::jsdoc_tag::JSDocTag;
use super::utils;

/// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
pub fn parse_jsdoc(source_text: &str) -> (String, Vec<JSDocTag>) {
    debug_assert!(!source_text.starts_with("/*"));
    debug_assert!(!source_text.ends_with("*/"));

    // JSDoc consists of comment and tags.
    // - Comment goes first, and tags(`@xxx`) follow
    // - Both can be optional
    // - Each tag is also separated by whitespace + `@`
    let mut comment = "";
    let mut tags = vec![];

    // So, find `@` to split comment and each tag.
    // But `@` can be found inside of `{}` (e.g. `{@see link}`), it should be distinguished.
    let mut in_braces = false;
    let mut comment_found = false;
    let (mut start, mut end) = (0, 0);
    for ch in source_text.chars() {
        match ch {
            '{' => in_braces = true,
            '}' => in_braces = false,
            '@' if !in_braces => {
                let part = &source_text[start..end];

                if comment_found {
                    tags.push(parse_jsdoc_tag(part));
                } else {
                    comment = part;
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

        if comment_found {
            tags.push(parse_jsdoc_tag(part));
        } else {
            comment = part;
        }
    }

    (utils::trim_comment(comment), tags)
}

// TODO: Manage `Span`
// - with (start, end) + global comment span.start
// - add kind only span?
/// tag_content: Starts with `@`, may be mulitline
fn parse_jsdoc_tag(tag_content: &str) -> JSDocTag {
    debug_assert!(tag_content.starts_with('@'));

    // This surely exists, at least `@` itself
    let (k_start, k_end) = utils::find_token_range(tag_content).unwrap();

    JSDocTag::new(
        // Omit the first `@`
        &tag_content[k_start + 1..k_end],
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
        // Includes splitter whitespace is needed to distinguish these cases.
        // If not included, both body_part will starts with `* <- ...`!
        // It will be trimmed later.
        &tag_content[k_end..],
    )
}

#[cfg(test)]
mod test {
    use super::parse_jsdoc;
    use super::parse_jsdoc_tag;
    use super::JSDocTag;

    fn parse_from_full_text(full_text: &str) -> (String, Vec<JSDocTag>) {
        // Outside of markers can be trimmed
        let source_text = full_text.trim().trim_start_matches("/**").trim_end_matches("*/");
        parse_jsdoc(source_text)
    }

    #[test]
    fn parses_jsdoc_comment() {
        assert_eq!(parse_jsdoc("hello source"), ("hello source".to_string(), vec![]));
        assert_eq!(
            parse_from_full_text("/** hello full_text */"),
            ("hello full_text".to_string(), vec![])
        );
        assert_eq!(parse_from_full_text("/***/"), (String::new(), vec![]));
        assert_eq!(parse_from_full_text("/****/"), ("*".to_string(), vec![]));
        assert_eq!(parse_from_full_text("/*****/"), ("**".to_string(), vec![]));
        assert_eq!(
            parse_from_full_text(
                "/**
                  * * x
                  ** y
                  */"
            )
            .0,
            "* x\n* y"
        );

        assert_eq!(parse_jsdoc(" <- trim -> ").0, "<- trim ->");
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
comment {@link link} ...
@x
*/"
            )
            .0,
            "this is\ncomment {@link link} ..."
        );
        assert_eq!(
            parse_from_full_text(
                "/**
　　　　　　　　　* 日本語とか
　　　　　　　　　* multibyte文字はどう⁉️
                  */"
            )
            .0,
            "日本語とか\nmultibyte文字はどう⁉️"
        );

        assert_eq!(
            parse_jsdoc("hello {@see inline} source {@a 2}").0,
            "hello {@see inline} source {@a 2}"
        );

        assert_eq!(parse_jsdoc("").0, "");
    }

    #[test]
    fn parses_single_line_1_jsdoc() {
        assert_eq!(parse_jsdoc("@deprecated"), parse_from_full_text("/** @deprecated*/"));
        assert_eq!(parse_jsdoc("@deprecated").1, vec![parse_jsdoc_tag("@deprecated")]);

        assert_eq!(parse_jsdoc("").1, vec![]);

        assert_eq!(
            parse_from_full_text("/**@foo since 2024 */").1,
            vec![parse_jsdoc_tag("@foo since 2024 ")]
        );
        assert_eq!(parse_from_full_text("/**@*/").1, vec![JSDocTag::new("", "")]);
    }

    #[test]
    fn parses_single_line_n_jsdocs() {
        assert_eq!(
            parse_from_full_text("/** @foo @bar */").1,
            vec![JSDocTag::new("foo", " "), JSDocTag::new("bar", " ")]
        );
        assert_eq!(
            parse_from_full_text("/** @aiue あいうえ @o お*/").1,
            vec![JSDocTag::new("aiue", " あいうえ "), JSDocTag::new("o", " お")]
        );
        assert_eq!(
            parse_from_full_text("/** @a @@ @d */").1,
            vec![
                JSDocTag::new("a", " "),
                JSDocTag::new("", ""),
                JSDocTag::new("", " "),
                JSDocTag::new("d", " ")
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
            vec![JSDocTag::new("yo", "\n    ")]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                    *     @foo
                          */"
            )
            .1,
            vec![JSDocTag::new("foo", "\n                          ")]
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
            vec![JSDocTag::new("x", " with asterisk\n         ")]
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
            vec![JSDocTag::new("y", " without\n        asterisk\n             ")]
        );
    }

    #[test]
    fn parses_multiline_n_jsdocs() {
        assert_eq!(
            parse_from_full_text(
                "
    /**
       @foo@bar
    * @baz
     */
            "
            )
            .1,
            vec![
                JSDocTag::new("foo", ""),
                JSDocTag::new("bar", "\n    * "),
                JSDocTag::new("baz", "\n     ")
            ]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                      * @one
                  *
                  * ...
              *
                      * @two */"
            )
            .1,
            vec![
                JSDocTag::new("one", "\n                  *\n                  * ...\n              *\n                      * "),
                JSDocTag::new("two", " "),
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
                JSDocTag::new(
                    "hey",
                    " you!\n                  *   Are you OK?\n                  * "
                ),
                JSDocTag::new("yes", " I'm fine\n                  ")
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
        let mut tags = jsdoc.1.iter();
        assert_eq!(tags.len(), 7);

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "export");
        assert_eq!(tag.comment(), "");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "template");
        assert_eq!(tag.comment(), "T");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(tag.type_name_comment(), (Some("*"), Some("data"), ": table data".to_string()));

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(
            tag.type_name_comment(),
            (Some("string"), Some("childrenColumnName"), ": 指定树形结构的列名".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(
            tag.type_name_comment(),
            (Some("Set<Key>"), Some("expandedKeys"), ": 展开的行对应的keys".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(
            tag.type_name_comment(),
            (Some("GetRowKey<T>"), Some("getRowKey"), ": 获取当前rowKey的方法".to_string())
        );

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "returns");
        assert_eq!(tag.type_comment(), (None, "flattened data".to_string()));
    }
}

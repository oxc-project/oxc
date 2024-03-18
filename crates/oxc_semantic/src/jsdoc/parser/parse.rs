use super::jsdoc_tag::JSDocTag;
use super::utils;

/// source_text: Inside of /**HERE*/, NOT includes `/**` and `*/`
pub fn parse_jsdoc(source_text: &str) -> (String, Vec<JSDocTag>) {
    let mut comment = String::new();
    let mut tags = vec![];

    // JSDoc consists of comment and tags.
    // - Comment goes first, and tags(`@xxx`) follow
    // - Each tag is also separated by whitespace + `@`
    // `@` can be inside of `{}` (e.g. `{@link}`) and it should be distinguished.
    let mut draft = String::new();
    let mut in_braces = false;
    let mut has_comment = false;
    for ch in source_text.chars() {
        match ch {
            '{' => in_braces = true,
            '}' => in_braces = false,
            '@' if !in_braces => {
                if has_comment {
                    tags.push(parse_jsdoc_tag(&draft.clone()));
                } else {
                    comment = draft.clone();
                    has_comment = true;
                }

                draft.clear();
            }
            _ => {}
        }

        draft.push(ch);
    }

    if !draft.is_empty() {
        if has_comment {
            tags.push(parse_jsdoc_tag(&draft.clone()));
        } else {
            comment = draft;
        }
    }

    (utils::trim_multiline_comment(&comment), tags)
}

/// tag_text: Starts with `@`, may be multiline
fn parse_jsdoc_tag(tag_text: &str) -> JSDocTag {
    let mut chars = tag_text.chars().skip(/* @ */ 1);

    let mut kind = String::new();
    for ch in chars.by_ref() {
        if ch == ' ' || ch == '\n' {
            break;
        }
        kind.push(ch);
    }

    // How to prase body is not determined yet, it depends on the use case!
    JSDocTag::new(kind, chars.collect())
}

#[cfg(test)]
mod test {
    use super::parse_jsdoc;
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
　　　　　　　　　* multibyte文字はどう？
                  */"
            )
            .0,
            "日本語とか\nmultibyte文字はどう？"
        );

        assert_eq!(parse_jsdoc("hello {@see inline} source").0, "hello {@see inline} source");
    }

    #[test]
    fn parses_single_line_1_jsdoc() {
        assert_eq!(parse_jsdoc("@deprecated"), parse_from_full_text("/** @deprecated */"));
        assert_eq!(
            parse_jsdoc("@deprecated").1,
            vec![JSDocTag::new("deprecated".to_string(), String::new())]
        );

        assert_eq!(
            parse_from_full_text("/**@foo since 2024 */").1,
            vec![JSDocTag::new("foo".to_string(), "since 2024 ".to_string())]
        );
        assert_eq!(
            parse_from_full_text("/**@*/").1,
            vec![JSDocTag::new(String::new(), String::new())]
        );
    }

    #[test]
    fn parses_single_line_n_jsdocs() {
        assert_eq!(
            parse_from_full_text("/** @foo @bar */").1,
            vec![
                JSDocTag::new("foo".to_string(), String::new()),
                JSDocTag::new("bar".to_string(), String::new()),
            ]
        );
        assert_eq!(
            parse_from_full_text("/** @aiue あいうえ @o お*/").1,
            vec![
                JSDocTag::new("aiue".to_string(), "あいうえ ".to_string()),
                JSDocTag::new("o".to_string(), "お".to_string()),
            ]
        );
        assert_eq!(
            parse_from_full_text("/** @a @@ @d */").1,
            vec![
                JSDocTag::new("a".to_string(), String::new()),
                JSDocTag::new(String::new(), String::new()),
                JSDocTag::new(String::new(), String::new()),
                JSDocTag::new("d".to_string(), String::new()),
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
            vec![JSDocTag::new("yo".to_string(), "    ".to_string())]
        );
        assert_eq!(
            parse_from_full_text(
                "/**
                    *     @foo
                          */"
            )
            .1,
            vec![JSDocTag::new("foo".to_string(), "                          ".to_string())]
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
            vec![JSDocTag::new("x".to_string(), "with asterisk\n         ".to_string())]
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
            vec![JSDocTag::new(
                "y".to_string(),
                "without\n        asterisk\n             ".to_string()
            )]
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
                JSDocTag::new("foo".to_string(), String::new()),
                JSDocTag::new("bar".to_string(), "    * ".to_string()),
                JSDocTag::new("baz".to_string(), "     ".to_string()),
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
                JSDocTag::new("one".to_string(), "                  *\n                  * ...\n              *\n                      * ".to_string()),
                JSDocTag::new("two".to_string(), String::new()),
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
                    "hey".to_string(),
                    "you!\n                  *   Are you OK?\n                  * ".to_string()
                ),
                JSDocTag::new("yes".to_string(), "I'm fine\n                  ".to_string())
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
        assert_eq!(tag.comment(), "{*} data : table data");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(tag.comment(), "{string} childrenColumnName : 指定树形结构的列名");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(tag.comment(), "{Set<Key>} expandedKeys : 展开的行对应的keys");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "param");
        assert_eq!(tag.comment(), "{GetRowKey<T>} getRowKey  : 获取当前rowKey的方法");

        let tag = tags.next().unwrap();
        assert_eq!(tag.kind, "returns");
        assert_eq!(tag.comment(), "flattened data");
    }
}

mod parser_impl;
mod reader;
mod span_factory;
mod state;
mod unicode;
mod unicode_property;

pub use parser_impl::Parser;

#[cfg(test)]
mod test {
    use crate::{Parser, ParserOptions};
    use oxc_allocator::Allocator;

    fn default() -> ParserOptions {
        ParserOptions::default()
    }
    fn with_unicode_mode() -> ParserOptions {
        ParserOptions { unicode_mode: true, ..Default::default() }
    }
    fn with_unicode_sets_mode() -> ParserOptions {
        ParserOptions { unicode_mode: true, unicode_sets_mode: true, ..Default::default() }
    }

    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("", default()),
            ("a", default()),
            ("a+", default()),
            ("a*", default()),
            ("a?", default()),
            ("^$^$^$", default()),
            ("(?=a){1}", default()),
            ("(?!a){1}", default()),
            ("a{1}", default()),
            ("a{1", default()),
            ("a|{", default()),
            ("a{", default()),
            ("a{,", default()),
            ("a{1,", default()),
            ("a{1,}", default()),
            ("a{1,2}", default()),
            ("x{9007199254740991}", default()),
            ("x{9007199254740991,9007199254740991}", default()),
            ("a|b", default()),
            ("a|b|c", default()),
            ("a|b+?|c", default()),
            ("a+b*?c{1}d{2,}e{3,4}?", default()),
            (r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$", default()),
            ("a.b..", default()),
            (r"\d\D\s\S\w\W", default()),
            (r"\x", default()),
            (r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{Basic_Emoji}", default()),
            (r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}", with_unicode_mode()),
            (r"^\p{General_Category=cntrl}+$", with_unicode_mode()),
            (r"\p{Basic_Emoji}", with_unicode_sets_mode()),
            (r"\n\cM\0\x41\u1f60\.\/", default()),
            (r"\c0", default()),
            (r"\0", default()),
            (r"\0", with_unicode_mode()),
            (r"\u", default()),
            (r"\u{", default()),
            (r"\u{}", default()),
            (r"\u{0}", default()),
            (r"\u{1f600}", default()),
            (r"\u{1f600}", with_unicode_mode()),
            ("(?:abc)", default()),
            (r"(?<\u{1d49c}>.)\x1f", default()),
            ("a]", default()),
            ("a}", default()),
            ("]", default()),
            ("[]", default()),
            ("[a]", default()),
            ("[ab]", default()),
            ("[a-b]", default()),
            ("[-]", default()),
            ("[a-]", default()),
            ("[-a]", default()),
            ("[-a-]", default()),
            (r"[a\-b]", default()),
            (r"[-a-b]", default()),
            (r"[a-b-]", default()),
            (r"[a\-b-]", default()),
            (r"[\[\]\-]", default()),
            ("[a-z0-9]", default()),
            ("[a-a]", default()),
            (r"[\d-\D]", default()),
            (r"^([\ud801[\udc28-\udc4f])$", default()),
            (r"[a-c]]", default()),
            (
                r"[ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ]",
                default(),
            ),
            (r"[a-z0-9[.\\]]", with_unicode_sets_mode()),
            (r"[a&&b&&c]", with_unicode_sets_mode()),
            (r"[a--b--c]", with_unicode_sets_mode()),
            (r"[[a-z]--b--c]", with_unicode_sets_mode()),
            (r"[[[[[[[[[[[[[[[[[[[[[[[[a]]]]]]]]]]]]]]]]]]]]]]]]", with_unicode_sets_mode()),
            (r"[\q{}\q{a}\q{bc}\q{d|e|f}\q{|||}]", with_unicode_sets_mode()),
            (r"(?<foo>A)\k<foo>", default()),
            (r"(?<!a>)\k<a>", default()),
            (r"\k", default()),
            (r"\k<4>", default()),
            (r"\k<a>", default()),
            (r"(?<a>)\k<a>", default()),
            (r"(?<a>)\k<a>", with_unicode_mode()),
            (r"\1", default()),
            (r"\1()", default()),
            (r"\1()", with_unicode_mode()),
            (r"(?<n1>..)(?<n2>..)", default()),
            // ES2025 ---
            // TODO: Duplicate named capturing groups
            // (r"(?<n1>..)|(?<n1>..)", default()),
            // (r"(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})", default()),
            // (r"(?:(?<a>x)|(?<a>y))\k<a>", default()),
            // Modifiers
            (r"(?:.)", default()),
            (r"(?s:.)", default()),
            (r"(?ism:.)", default()),
            (r"(?-s:.)", default()),
            (r"(?-smi:.)", default()),
            (r"(?s-im:.)", default()),
            (r"(?si-m:.)", default()),
            (r"(?im-s:.)", with_unicode_sets_mode()),
            (r"(?ims-:.)", default()),
        ] {
            let res = Parser::new(&allocator, source_text, *options).parse();
            if let Err(err) = res {
                panic!("Failed to parse {source_text} with {options:?}\n💥 {err}");
            }
        }
    }

    #[test]
    fn should_fail() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("a)", default()),
            (r"a\", default()),
            ("a]", with_unicode_mode()),
            ("a}", with_unicode_mode()),
            ("a|+", default()),
            ("a|{", with_unicode_mode()),
            ("a{", with_unicode_mode()),
            ("a{1", with_unicode_mode()),
            ("a{1,", with_unicode_mode()),
            ("a{,", with_unicode_mode()),
            ("x{9007199254740992}", default()),
            ("x{9007199254740991,9007199254740992}", default()),
            ("x{99999999999999999999999999999999999999999999999999}", default()),
            (r"\99999999999999999999999999999999999999999999999999", default()),
            (r"\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}", with_unicode_mode()),
            ("(?=a", default()),
            ("(?<!a", default()),
            (r"\c0", with_unicode_mode()),
            (r"\xa", with_unicode_mode()),
            (r"a\u", with_unicode_mode()),
            (r"\p{Emoji_Presentation", with_unicode_mode()),
            (r"\p{Script=", with_unicode_mode()),
            (r"\ka", with_unicode_mode()),
            (r"\k", with_unicode_mode()),
            (r"\k<", with_unicode_mode()),
            (r"\k<>", with_unicode_mode()),
            (r"\k<4>", with_unicode_mode()),
            (r"\k<a", with_unicode_mode()),
            (r"\1", with_unicode_mode()),
            (r"\k<a>", with_unicode_mode()),
            ("a(?:", default()),
            ("(a", default()),
            ("(?<a>", default()),
            ("(?<", default()),
            (r"(?<a\>.)", default()),
            (r"(?<a\>.)", with_unicode_mode()),
            (r"(?<\>.)", default()),
            (r"(?<\>.)", with_unicode_mode()),
            ("(?)", default()),
            ("(?=a){1}", with_unicode_mode()),
            ("(?!a){1}", with_unicode_mode()),
            (r"[\d-\D]", with_unicode_mode()),
            ("[", default()),
            ("[", with_unicode_sets_mode()),
            ("[[", with_unicode_sets_mode()),
            ("[[]", with_unicode_sets_mode()),
            ("[z-a]", default()),
            (r"[a-c]]", with_unicode_mode()),
            (
                r"^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$",
                default(),
            ),
            (r"[[\d-\D]]", with_unicode_sets_mode()),
            (r"[a&&b--c]", with_unicode_sets_mode()),
            (r"[a--b&&c]", with_unicode_sets_mode()),
            (r"[\q{]", with_unicode_sets_mode()),
            (r"[\q{\a}]", with_unicode_sets_mode()),
            // ES2025 ---
            // TODO: Duplicate named capturing groups
            (r"(?<n>..)|(?<n>..)", default()), // This will be valid
            // (r"(?<a>|(?<a>))", default()), // Nested, still invalid
            // Modifiers
            (r"(?a:.)", default()),
            (r"(?-S:.)", default()),
            (r"(?-:.)", default()),
            (r"(?iM:.)", default()),
            (r"(?imms:.)", default()),
            (r"(?-sI:.)", default()),
            (r"(?ii-s:.)", default()),
            (r"(?i-msm:.)", default()),
            (r"(?i", default()),
            (r"(?i-", default()),
            (r"(?i-s", default()),
        ] {
            assert!(
                Parser::new(&allocator, source_text, *options).parse().is_err(),
                "{source_text} should fail to parse with {options:?}, but passed!"
            );
        }
    }

    #[test]
    fn should_fail_early_errors() {
        let allocator = Allocator::default();

        for (source_text, options, is_err) in &[
            // No tests for 4,294,967,295 left parens
            (r"(?<n>..)(?<n>..)", default(), true),
            (r"a{2,1}", default(), true),
            (r"(?<a>)\k<n>", default(), true),
            (r"()\2", with_unicode_mode(), true),
            (r"[a-\d]", with_unicode_mode(), true),
            (r"[\d-z]", with_unicode_mode(), true),
            (r"[\d-\d]", with_unicode_mode(), true),
            (r"[z-a]", default(), true),
            (r"\u{110000}", with_unicode_mode(), true),
            (r"(?<\uD800\uDBFF>)", default(), true),
            (r"\u{0}\u{110000}", with_unicode_mode(), true),
            (r"(?<a\uD800\uDBFF>)", default(), true),
            (r"\p{Foo=Bar}", with_unicode_mode(), true),
            (r"\p{Foo}", with_unicode_mode(), true),
            (r"\p{Basic_Emoji}", with_unicode_mode(), true),
            (r"\P{Basic_Emoji}", with_unicode_sets_mode(), true),
            (r"[^\p{Basic_Emoji}]", with_unicode_sets_mode(), true),
            (r"[[^\p{Basic_Emoji}]]", with_unicode_sets_mode(), true),
            (r"[^\q{}]", with_unicode_sets_mode(), true),
            (r"[[^\q{}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{ng}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{a|}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{ng}\q{o|k}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}\q{ng}\q{o|k}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}\q{o|k}\q{ng}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{}&&\q{ng}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{ng}&&\q{o|k}]]", with_unicode_sets_mode(), false),
            (r"[[^\q{ng}&&\q{o|k}&&\q{ng}]]", with_unicode_sets_mode(), false),
            (r"[[^\q{ng}--\q{o|k}]]", with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}--\q{ng}]]", with_unicode_sets_mode(), false),
            (r"[[z-a]]", with_unicode_sets_mode(), true),
            (r"[[[[[^[[[[\q{ng}]]]]]]]]]", with_unicode_sets_mode(), true),
            (r"[^[[[[[[[[[[[[[[[[\q{ng}]]]]]]]]]]]]]]]]]", with_unicode_sets_mode(), true),
            // ES2025 ---
            // Modifiers
            (r"(?ii:.)", default(), true),
            (r"(?-ss:.)", default(), true),
            (r"(?im-im:.)", default(), true),
        ] {
            assert_eq!(
                Parser::new(&allocator, source_text, *options).parse().is_err(),
                *is_err,
                "{source_text} should fail with early error with {options:?}, but passed!"
            );
        }
    }

    #[test]
    fn should_handle_empty() {
        let allocator = Allocator::default();
        let pattern1 = Parser::new(&allocator, "", default()).parse().unwrap();
        let pattern2 = Parser::new(
            &allocator,
            "''",
            ParserOptions { parse_string_literal: true, ..default() },
        )
        .parse()
        .unwrap();

        assert_eq!(pattern1.body.body[0].body.len(), 1);
        assert_eq!(pattern2.body.body[0].body.len(), 1);
    }

    #[test]
    fn should_handle_unicode() {
        let allocator = Allocator::default();
        let source_text = "このEmoji🥹の数が変わる";

        for (options, expected) in
            &[(default(), 15), (with_unicode_mode(), 14), (with_unicode_sets_mode(), 14)]
        {
            let pattern = Parser::new(&allocator, source_text, *options).parse().unwrap();
            assert_eq!(pattern.body.body[0].body.len(), *expected);
        }
    }

    #[test]
    fn span_offset() {
        let allocator = Allocator::default();

        let source_text = "Adjust span but should have no side effect for parsing";
        let ret1 = Parser::new(
            &allocator,
            source_text,
            ParserOptions { span_offset: 0, ..ParserOptions::default() },
        )
        .parse()
        .unwrap();
        let ret2 = Parser::new(
            &allocator,
            source_text,
            ParserOptions { span_offset: 10, ..ParserOptions::default() },
        )
        .parse()
        .unwrap();

        assert_ne!(ret1.span, ret2.span);
        assert_eq!(ret1.to_string(), ret2.to_string());
    }
}

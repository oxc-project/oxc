mod parser;
mod reader;
mod state;
mod unicode;
mod unicode_property;

pub use parser::PatternParser;

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{ast::RegularExpressionFlags, PatternParser, PatternParserOptions};

    const DEFAULT_OPTIONS: PatternParserOptions =
        PatternParserOptions { span_offset: 0, flags: RegularExpressionFlags::empty() };
    const UNICODE_OPTIONS: PatternParserOptions =
        PatternParserOptions { span_offset: 0, flags: RegularExpressionFlags::U };
    const UNICODE_SET_OPTIONS: PatternParserOptions =
        PatternParserOptions { span_offset: 0, flags: RegularExpressionFlags::V };

    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("", DEFAULT_OPTIONS),
            ("a", DEFAULT_OPTIONS),
            ("a+", DEFAULT_OPTIONS),
            ("a*", DEFAULT_OPTIONS),
            ("a?", DEFAULT_OPTIONS),
            ("^$^$^$", DEFAULT_OPTIONS),
            ("(?=a){1}", DEFAULT_OPTIONS),
            ("(?!a){1}", DEFAULT_OPTIONS),
            ("a{1}", DEFAULT_OPTIONS),
            ("a{1", DEFAULT_OPTIONS),
            ("a|{", DEFAULT_OPTIONS),
            ("a{", DEFAULT_OPTIONS),
            ("a{,", DEFAULT_OPTIONS),
            ("a{1,", DEFAULT_OPTIONS),
            ("a{1,}", DEFAULT_OPTIONS),
            ("a{1,2}", DEFAULT_OPTIONS),
            ("x{9007199254740991}", DEFAULT_OPTIONS),
            ("x{9007199254740991,9007199254740991}", DEFAULT_OPTIONS),
            ("a|b", DEFAULT_OPTIONS),
            ("a|b|c", DEFAULT_OPTIONS),
            ("a|b+?|c", DEFAULT_OPTIONS),
            ("a+b*?c{1}d{2,}e{3,4}?", DEFAULT_OPTIONS),
            (r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$", DEFAULT_OPTIONS),
            ("a.b..", DEFAULT_OPTIONS),
            (r"\d\D\s\S\w\W", DEFAULT_OPTIONS),
            (r"\x", DEFAULT_OPTIONS),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{Basic_Emoji}",
                DEFAULT_OPTIONS,
            ),
            (r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}", UNICODE_OPTIONS),
            (r"^\p{General_Category=cntrl}+$", UNICODE_OPTIONS),
            (r"\p{Basic_Emoji}", UNICODE_SET_OPTIONS),
            (r"\n\cM\0\x41\u1f60\.\/", DEFAULT_OPTIONS),
            (r"\c0", DEFAULT_OPTIONS),
            (r"\0", DEFAULT_OPTIONS),
            (r"\0", UNICODE_OPTIONS),
            (r"\u", DEFAULT_OPTIONS),
            (r"\u{", DEFAULT_OPTIONS),
            (r"\u{}", DEFAULT_OPTIONS),
            (r"\u{0}", DEFAULT_OPTIONS),
            (r"\u{1f600}", DEFAULT_OPTIONS),
            (r"\u{1f600}", UNICODE_OPTIONS),
            ("(?:abc)", DEFAULT_OPTIONS),
            (r"(?<\u{1d49c}>.)\x1f", DEFAULT_OPTIONS),
            ("a]", DEFAULT_OPTIONS),
            ("a}", DEFAULT_OPTIONS),
            ("]", DEFAULT_OPTIONS),
            ("[]", DEFAULT_OPTIONS),
            ("[a]", DEFAULT_OPTIONS),
            ("[ab]", DEFAULT_OPTIONS),
            ("[a-b]", DEFAULT_OPTIONS),
            ("[-]", DEFAULT_OPTIONS),
            ("[a-]", DEFAULT_OPTIONS),
            ("[-a]", DEFAULT_OPTIONS),
            ("[-a-]", DEFAULT_OPTIONS),
            (r"[a\-b]", DEFAULT_OPTIONS),
            (r"[-a-b]", DEFAULT_OPTIONS),
            (r"[a-b-]", DEFAULT_OPTIONS),
            (r"[a\-b-]", DEFAULT_OPTIONS),
            (r"[\[\]\-]", DEFAULT_OPTIONS),
            ("[a-z0-9]", DEFAULT_OPTIONS),
            ("[a-a]", DEFAULT_OPTIONS),
            (r"[\d-\D]", DEFAULT_OPTIONS),
            (r"^([\ud801[\udc28-\udc4f])$", DEFAULT_OPTIONS),
            (r"[a-c]]", DEFAULT_OPTIONS),
            (
                r"[ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ]",
                DEFAULT_OPTIONS,
            ),
            (r"[a-z0-9[.\\]]", UNICODE_SET_OPTIONS),
            (r"[a&&b&&c]", UNICODE_SET_OPTIONS),
            (r"[a--b--c]", UNICODE_SET_OPTIONS),
            (r"[[a-z]--b--c]", UNICODE_SET_OPTIONS),
            (r"[[[[[[[[[[[[[[[[[[[[[[[[a]]]]]]]]]]]]]]]]]]]]]]]]", UNICODE_SET_OPTIONS),
            (r"[\q{}\q{a}\q{bc}\q{d|e|f}\q{|||}]", UNICODE_SET_OPTIONS),
            (r"(?<foo>A)\k<foo>", DEFAULT_OPTIONS),
            (r"(?<!a>)\k<a>", DEFAULT_OPTIONS),
            (r"\k", DEFAULT_OPTIONS),
            (r"\k<4>", DEFAULT_OPTIONS),
            (r"\k<a>", DEFAULT_OPTIONS),
            (r"(?<a>)\k<a>", DEFAULT_OPTIONS),
            (r"(?<a>)\k<a>", UNICODE_OPTIONS),
            (r"\1", DEFAULT_OPTIONS),
            (r"\1()", DEFAULT_OPTIONS),
            (r"\1()", UNICODE_OPTIONS),
            (r"(?<n1>..)(?<n2>..)", DEFAULT_OPTIONS),
            // TODO: ES2025 Duplicate named capturing groups
            // (r"(?<n1>..)|(?<n1>..)", DEFAULT_OPTIONS),
            // (r"(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})", DEFAULT_OPTIONS),
            // (r"(?:(?<a>x)|(?<a>y))\k<a>", DEFAULT_OPTIONS),
        ] {
            let res = PatternParser::new(&allocator, source_text, *options).parse();
            if let Err(err) = res {
                panic!("Failed to parse {source_text} with {options:?}\n💥 {err}");
            }
        }
    }

    #[test]
    fn should_fail() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("a)", DEFAULT_OPTIONS),
            (r"a\", DEFAULT_OPTIONS),
            ("a]", UNICODE_OPTIONS),
            ("a}", UNICODE_OPTIONS),
            ("a|+", DEFAULT_OPTIONS),
            ("a|{", UNICODE_OPTIONS),
            ("a{", UNICODE_OPTIONS),
            ("a{1", UNICODE_OPTIONS),
            ("a{1,", UNICODE_OPTIONS),
            ("a{,", UNICODE_OPTIONS),
            ("x{9007199254740992}", DEFAULT_OPTIONS),
            ("x{9007199254740991,9007199254740992}", DEFAULT_OPTIONS),
            ("x{99999999999999999999999999999999999999999999999999}", DEFAULT_OPTIONS),
            (r"\99999999999999999999999999999999999999999999999999", DEFAULT_OPTIONS),
            (r"\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}", UNICODE_OPTIONS),
            ("(?=a", DEFAULT_OPTIONS),
            ("(?<!a", DEFAULT_OPTIONS),
            (r"\c0", UNICODE_OPTIONS),
            (r"\xa", UNICODE_OPTIONS),
            (r"a\u", UNICODE_OPTIONS),
            (r"\p{Emoji_Presentation", UNICODE_OPTIONS),
            (r"\p{Script=", UNICODE_OPTIONS),
            (r"\ka", UNICODE_OPTIONS),
            (r"\k", UNICODE_OPTIONS),
            (r"\k<", UNICODE_OPTIONS),
            (r"\k<>", UNICODE_OPTIONS),
            (r"\k<4>", UNICODE_OPTIONS),
            (r"\k<a", UNICODE_OPTIONS),
            (r"\1", UNICODE_OPTIONS),
            (r"\k<a>", UNICODE_OPTIONS),
            ("a(?:", DEFAULT_OPTIONS),
            ("(a", DEFAULT_OPTIONS),
            ("(?<a>", DEFAULT_OPTIONS),
            (r"(?<a\>.)", DEFAULT_OPTIONS),
            (r"(?<a\>.)", UNICODE_OPTIONS),
            (r"(?<\>.)", DEFAULT_OPTIONS),
            (r"(?<\>.)", UNICODE_OPTIONS),
            ("(?)", DEFAULT_OPTIONS),
            ("(?=a){1}", UNICODE_OPTIONS),
            ("(?!a){1}", UNICODE_OPTIONS),
            (r"[\d-\D]", UNICODE_OPTIONS),
            ("[", DEFAULT_OPTIONS),
            ("[", UNICODE_SET_OPTIONS),
            ("[[", UNICODE_SET_OPTIONS),
            ("[[]", UNICODE_SET_OPTIONS),
            ("[z-a]", DEFAULT_OPTIONS),
            (r"[a-c]]", UNICODE_OPTIONS),
            (
                r"^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$",
                DEFAULT_OPTIONS,
            ),
            (r"[[\d-\D]]", UNICODE_SET_OPTIONS),
            (r"[a&&b--c]", UNICODE_SET_OPTIONS),
            (r"[a--b&&c]", UNICODE_SET_OPTIONS),
            (r"[\q{]", UNICODE_SET_OPTIONS),
            (r"[\q{\a}]", UNICODE_SET_OPTIONS),
            // TODO: ES2025 Duplicate named capturing groups
            (r"(?<n>..)|(?<n>..)", DEFAULT_OPTIONS), // This will be valid
                                                     // (r"(?<a>|(?<a>))", DEFAULT_OPTIONS), // Nested, still invalid
        ] {
            assert!(
                PatternParser::new(&allocator, source_text, *options).parse().is_err(),
                "{source_text} should fail to parse with {options:?}!"
            );
        }
    }

    #[test]
    fn should_fail_early_errors() {
        let allocator = Allocator::default();

        for (source_text, options, is_err) in &[
            // No tests for 4,294,967,295 left parens
            (r"(?<n>..)(?<n>..)", DEFAULT_OPTIONS, true),
            (r"a{2,1}", DEFAULT_OPTIONS, true),
            (r"(?<a>)\k<n>", DEFAULT_OPTIONS, true),
            (r"()\2", UNICODE_OPTIONS, true),
            (r"[a-\d]", UNICODE_OPTIONS, true),
            (r"[\d-z]", UNICODE_OPTIONS, true),
            (r"[\d-\d]", UNICODE_OPTIONS, true),
            (r"[z-a]", DEFAULT_OPTIONS, true),
            (r"\u{110000}", UNICODE_OPTIONS, true),
            (r"(?<\uD800\uDBFF>)", DEFAULT_OPTIONS, true),
            (r"\u{0}\u{110000}", UNICODE_OPTIONS, true),
            (r"(?<a\uD800\uDBFF>)", DEFAULT_OPTIONS, true),
            (r"\p{Foo=Bar}", UNICODE_OPTIONS, true),
            (r"\p{Foo}", UNICODE_OPTIONS, true),
            (r"\p{Basic_Emoji}", UNICODE_OPTIONS, true),
            (r"\P{Basic_Emoji}", UNICODE_SET_OPTIONS, true),
            (r"[^\p{Basic_Emoji}]", UNICODE_SET_OPTIONS, true),
            (r"[[^\p{Basic_Emoji}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{ng}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{a|}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{ng}\q{o|k}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{o|k}\q{ng}\q{o|k}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{o|k}\q{o|k}\q{ng}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{}&&\q{ng}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{ng}&&\q{o|k}]]", UNICODE_SET_OPTIONS, false),
            (r"[[^\q{ng}&&\q{o|k}&&\q{ng}]]", UNICODE_SET_OPTIONS, false),
            (r"[[^\q{ng}--\q{o|k}]]", UNICODE_SET_OPTIONS, true),
            (r"[[^\q{o|k}--\q{ng}]]", UNICODE_SET_OPTIONS, false),
            (r"[[z-a]]", UNICODE_SET_OPTIONS, true),
        ] {
            assert_eq!(
                PatternParser::new(&allocator, source_text, *options).parse().is_err(),
                *is_err,
                "{source_text} should early error with {options:?}!"
            );
        }
    }

    #[test]
    fn should_handle_empty() {
        let allocator = Allocator::default();
        let pattern = PatternParser::new(&allocator, "", DEFAULT_OPTIONS).parse().unwrap();

        assert_eq!(pattern.body.body[0].body.len(), 1);
    }

    #[test]
    fn should_handle_unicode() {
        let allocator = Allocator::default();
        let source_text = "このEmoji🥹の数が変わる";

        for (options, expected) in &[
            (DEFAULT_OPTIONS, 15),
            (DEFAULT_OPTIONS.with_flags(RegularExpressionFlags::U), 14),
            (DEFAULT_OPTIONS.with_flags(RegularExpressionFlags::V), 14),
        ] {
            let pattern = PatternParser::new(&allocator, source_text, *options).parse().unwrap();
            assert_eq!(pattern.body.body[0].body.len(), *expected);
        }
    }
}

mod parser;
mod reader;
mod state;
mod unicode;
mod unicode_property;

pub use parser::PatternParser;

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use crate::{ParserOptions, PatternParser};

    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("", ParserOptions::default()),
            ("a", ParserOptions::default()),
            ("a+", ParserOptions::default()),
            ("a*", ParserOptions::default()),
            ("a?", ParserOptions::default()),
            ("^$^$^$", ParserOptions::default()),
            ("(?=a){1}", ParserOptions::default()),
            ("(?!a){1}", ParserOptions::default()),
            ("a{1}", ParserOptions::default()),
            ("a{1", ParserOptions::default()),
            ("a|{", ParserOptions::default()),
            ("a{", ParserOptions::default()),
            ("a{,", ParserOptions::default()),
            ("a{1,", ParserOptions::default()),
            ("a{1,}", ParserOptions::default()),
            ("a{1,2}", ParserOptions::default()),
            ("x{9007199254740991}", ParserOptions::default()),
            ("x{9007199254740991,9007199254740991}", ParserOptions::default()),
            ("a|b", ParserOptions::default()),
            ("a|b|c", ParserOptions::default()),
            ("a|b+?|c", ParserOptions::default()),
            ("a+b*?c{1}d{2,}e{3,4}?", ParserOptions::default()),
            (r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$", ParserOptions::default()),
            ("a.b..", ParserOptions::default()),
            (r"\d\D\s\S\w\W", ParserOptions::default()),
            (r"\x", ParserOptions::default()),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{Basic_Emoji}",
                ParserOptions::default(),
            ),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}",
                ParserOptions::default().with_unicode_mode(),
            ),
            (r"^\p{General_Category=cntrl}+$", ParserOptions::default().with_unicode_mode()),
            (r"\p{Basic_Emoji}", ParserOptions::default().with_unicode_sets_mode()),
            (r"\n\cM\0\x41\u1f60\.\/", ParserOptions::default()),
            (r"\c0", ParserOptions::default()),
            (r"\0", ParserOptions::default()),
            (r"\0", ParserOptions::default().with_unicode_mode()),
            (r"\u", ParserOptions::default()),
            (r"\u{", ParserOptions::default()),
            (r"\u{}", ParserOptions::default()),
            (r"\u{0}", ParserOptions::default()),
            (r"\u{1f600}", ParserOptions::default()),
            (r"\u{1f600}", ParserOptions::default().with_unicode_mode()),
            ("(?:abc)", ParserOptions::default()),
            (r"(?<\u{1d49c}>.)\x1f", ParserOptions::default()),
            ("a]", ParserOptions::default()),
            ("a}", ParserOptions::default()),
            ("]", ParserOptions::default()),
            ("[]", ParserOptions::default()),
            ("[a]", ParserOptions::default()),
            ("[ab]", ParserOptions::default()),
            ("[a-b]", ParserOptions::default()),
            ("[-]", ParserOptions::default()),
            ("[a-]", ParserOptions::default()),
            ("[-a]", ParserOptions::default()),
            ("[-a-]", ParserOptions::default()),
            (r"[a\-b]", ParserOptions::default()),
            (r"[-a-b]", ParserOptions::default()),
            (r"[a-b-]", ParserOptions::default()),
            (r"[a\-b-]", ParserOptions::default()),
            (r"[\[\]\-]", ParserOptions::default()),
            ("[a-z0-9]", ParserOptions::default()),
            ("[a-a]", ParserOptions::default()),
            (r"[\d-\D]", ParserOptions::default()),
            (r"^([\ud801[\udc28-\udc4f])$", ParserOptions::default()),
            (r"[a-c]]", ParserOptions::default()),
            (
                r"[ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ]",
                ParserOptions::default(),
            ),
            (r"[a-z0-9[.\\]]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[a&&b&&c]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[a--b--c]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[[a-z]--b--c]", ParserOptions::default().with_unicode_sets_mode()),
            (
                r"[[[[[[[[[[[[[[[[[[[[[[[[a]]]]]]]]]]]]]]]]]]]]]]]]",
                ParserOptions::default().with_unicode_sets_mode(),
            ),
            (
                r"[\q{}\q{a}\q{bc}\q{d|e|f}\q{|||}]",
                ParserOptions::default().with_unicode_sets_mode(),
            ),
            (r"(?<foo>A)\k<foo>", ParserOptions::default()),
            (r"(?<!a>)\k<a>", ParserOptions::default()),
            (r"\k", ParserOptions::default()),
            (r"\k<4>", ParserOptions::default()),
            (r"\k<a>", ParserOptions::default()),
            (r"(?<a>)\k<a>", ParserOptions::default()),
            (r"(?<a>)\k<a>", ParserOptions::default().with_unicode_mode()),
            (r"\1", ParserOptions::default()),
            (r"\1()", ParserOptions::default()),
            (r"\1()", ParserOptions::default().with_unicode_mode()),
            (r"(?<n1>..)(?<n2>..)", ParserOptions::default()),
            // TODO: ES2025 Duplicate named capturing groups
            // (r"(?<n1>..)|(?<n1>..)", ParserOptions::default()),
            // (r"(?<year>[0-9]{4})-[0-9]{2}|[0-9]{2}-(?<year>[0-9]{4})", ParserOptions::default()),
            // (r"(?:(?<a>x)|(?<a>y))\k<a>", ParserOptions::default()),
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
            ("a)", ParserOptions::default()),
            (r"a\", ParserOptions::default()),
            ("a]", ParserOptions::default().with_unicode_mode()),
            ("a}", ParserOptions::default().with_unicode_mode()),
            ("a|+", ParserOptions::default()),
            ("a|{", ParserOptions::default().with_unicode_mode()),
            ("a{", ParserOptions::default().with_unicode_mode()),
            ("a{1", ParserOptions::default().with_unicode_mode()),
            ("a{1,", ParserOptions::default().with_unicode_mode()),
            ("a{,", ParserOptions::default().with_unicode_mode()),
            ("x{9007199254740992}", ParserOptions::default()),
            ("x{9007199254740991,9007199254740992}", ParserOptions::default()),
            ("x{99999999999999999999999999999999999999999999999999}", ParserOptions::default()),
            (r"\99999999999999999999999999999999999999999999999999", ParserOptions::default()),
            (r"\u{FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF}", ParserOptions::default().with_unicode_mode()),
            ("(?=a", ParserOptions::default()),
            ("(?<!a", ParserOptions::default()),
            (r"\c0", ParserOptions::default().with_unicode_mode()),
            (r"\xa", ParserOptions::default().with_unicode_mode()),
            (r"a\u", ParserOptions::default().with_unicode_mode()),
            (r"\p{Emoji_Presentation", ParserOptions::default().with_unicode_mode()),
            (r"\p{Script=", ParserOptions::default().with_unicode_mode()),
            (r"\ka", ParserOptions::default().with_unicode_mode()),
            (r"\k", ParserOptions::default().with_unicode_mode()),
            (r"\k<", ParserOptions::default().with_unicode_mode()),
            (r"\k<>", ParserOptions::default().with_unicode_mode()),
            (r"\k<4>", ParserOptions::default().with_unicode_mode()),
            (r"\k<a", ParserOptions::default().with_unicode_mode()),
            (r"\1", ParserOptions::default().with_unicode_mode()),
            (r"\k<a>", ParserOptions::default().with_unicode_mode()),
            ("a(?:", ParserOptions::default()),
            ("(a", ParserOptions::default()),
            ("(?<a>", ParserOptions::default()),
            (r"(?<a\>.)", ParserOptions::default()),
            (r"(?<a\>.)", ParserOptions::default().with_unicode_mode()),
            (r"(?<\>.)", ParserOptions::default()),
            (r"(?<\>.)", ParserOptions::default().with_unicode_mode()),
            ("(?)", ParserOptions::default()),
            ("(?=a){1}", ParserOptions::default().with_unicode_mode()),
            ("(?!a){1}", ParserOptions::default().with_unicode_mode()),
            (r"[\d-\D]", ParserOptions::default().with_unicode_mode()),
            ("[", ParserOptions::default()),
            ("[", ParserOptions::default().with_unicode_sets_mode()),
            ("[[", ParserOptions::default().with_unicode_sets_mode()),
            ("[[]", ParserOptions::default().with_unicode_sets_mode()),
            ("[z-a]", ParserOptions::default()),
            (r"[a-c]]", ParserOptions::default().with_unicode_mode()),
            (
                r"^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$",
                ParserOptions::default(),
            ),
            (r"[[\d-\D]]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[a&&b--c]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[a--b&&c]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[\q{]", ParserOptions::default().with_unicode_sets_mode()),
            (r"[\q{\a}]", ParserOptions::default().with_unicode_sets_mode()),
            // TODO: ES2025 Duplicate named capturing groups
            (r"(?<n>..)|(?<n>..)", ParserOptions::default()), // This will be valid
                                                              // (r"(?<a>|(?<a>))", ParserOptions::default()), // Nested, still invalid
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
            (r"(?<n>..)(?<n>..)", ParserOptions::default(), true),
            (r"a{2,1}", ParserOptions::default(), true),
            (r"(?<a>)\k<n>", ParserOptions::default(), true),
            (r"()\2", ParserOptions::default().with_unicode_mode(), true),
            (r"[a-\d]", ParserOptions::default().with_unicode_mode(), true),
            (r"[\d-z]", ParserOptions::default().with_unicode_mode(), true),
            (r"[\d-\d]", ParserOptions::default().with_unicode_mode(), true),
            (r"[z-a]", ParserOptions::default(), true),
            (r"\u{110000}", ParserOptions::default().with_unicode_mode(), true),
            (r"(?<\uD800\uDBFF>)", ParserOptions::default(), true),
            (r"\u{0}\u{110000}", ParserOptions::default().with_unicode_mode(), true),
            (r"(?<a\uD800\uDBFF>)", ParserOptions::default(), true),
            (r"\p{Foo=Bar}", ParserOptions::default().with_unicode_mode(), true),
            (r"\p{Foo}", ParserOptions::default().with_unicode_mode(), true),
            (r"\p{Basic_Emoji}", ParserOptions::default().with_unicode_mode(), true),
            (r"\P{Basic_Emoji}", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[^\p{Basic_Emoji}]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\p{Basic_Emoji}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[^\q{}]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{ng}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{a|}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{ng}\q{o|k}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}\q{ng}\q{o|k}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}\q{o|k}\q{ng}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{}&&\q{ng}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{ng}&&\q{o|k}]]", ParserOptions::default().with_unicode_sets_mode(), false),
            (
                r"[[^\q{ng}&&\q{o|k}&&\q{ng}]]",
                ParserOptions::default().with_unicode_sets_mode(),
                false,
            ),
            (r"[[^\q{ng}--\q{o|k}]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[^\q{o|k}--\q{ng}]]", ParserOptions::default().with_unicode_sets_mode(), false),
            (r"[[z-a]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (r"[[[[[^[[[[\q{ng}]]]]]]]]]", ParserOptions::default().with_unicode_sets_mode(), true),
            (
                r"[^[[[[[[[[[[[[[[[[\q{ng}]]]]]]]]]]]]]]]]]",
                ParserOptions::default().with_unicode_sets_mode(),
                true,
            ),
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
        let pattern = PatternParser::new(&allocator, "", ParserOptions::default()).parse().unwrap();

        assert_eq!(pattern.body.body[0].body.len(), 1);
    }

    #[test]
    fn should_handle_unicode() {
        let allocator = Allocator::default();
        let source_text = "このEmoji🥹の数が変わる";

        for (options, expected) in &[
            (ParserOptions::default(), 15),
            (ParserOptions::default().with_unicode_mode(), 14),
            (ParserOptions::default().with_unicode_sets_mode(), 14),
        ] {
            let pattern = PatternParser::new(&allocator, source_text, *options).parse().unwrap();
            assert_eq!(pattern.body.body[0].body.len(), *expected);
        }
    }
}

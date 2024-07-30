mod parse;

pub use parse::PatternParser;

#[cfg(test)]
mod test {
    use crate::{ParserOptions, PatternParser};
    use oxc_allocator::Allocator;

    // NOTE: These may be useless when integlation tests are added
    #[test]
    fn should_pass() {
        let allocator = Allocator::default();

        for (source_text, options) in &[
            ("", ParserOptions::default()),
            ("a", ParserOptions::default()),
            ("a+", ParserOptions::default()),
            ("a*", ParserOptions::default()),
            ("a?", ParserOptions::default()),
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
            ("a|b", ParserOptions::default()),
            ("a|b|c", ParserOptions::default()),
            ("a|b+?|c", ParserOptions::default()),
            ("a+b*?c{1}d{2,}e{3,4}?", ParserOptions::default()),
            (r"^(?=ab)\b(?!cd)(?<=ef)\B(?<!gh)$", ParserOptions::default()),
            ("a.b..", ParserOptions::default()),
            (r"\d\D\s\S\w\W", ParserOptions::default()),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}",
                ParserOptions::default(),
            ),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}",
                ParserOptions::default().with_unicode_flags(true, false),
            ),
            (
                r"\p{Emoji_Presentation}\P{Script_Extensions=Latin}\p{Sc}|\p{P}",
                ParserOptions::default().with_unicode_flags(true, true),
            ),
            (r"\n\cM\0\x41\u1f60\.\/", ParserOptions::default()),
            (r"\u", ParserOptions::default()),
            (r"\u{", ParserOptions::default()),
            (r"\u{}", ParserOptions::default()),
            (r"\u{0}", ParserOptions::default()),
            (r"\u{1f600}", ParserOptions::default()),
            (r"\u{1f600}", ParserOptions::default().with_unicode_flags(true, false)),
            ("(?:abc)", ParserOptions::default()),
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
            (r"[ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ]", ParserOptions::default()),
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
            ("a]", ParserOptions::default().with_unicode_flags(true, false)),
            ("a}", ParserOptions::default().with_unicode_flags(true, false)),
            ("a|+", ParserOptions::default()),
            ("a|{", ParserOptions::default().with_unicode_flags(true, false)),
            ("a{", ParserOptions::default().with_unicode_flags(true, false)),
            ("a{1", ParserOptions::default().with_unicode_flags(true, false)),
            ("a{1,", ParserOptions::default().with_unicode_flags(true, false)),
            ("a{,", ParserOptions::default().with_unicode_flags(true, false)),
            ("a{2,1}", ParserOptions::default()),
            ("(?=a", ParserOptions::default()),
            ("(?<!a", ParserOptions::default()),
            (r"\xa", ParserOptions::default()),
            (r"a\u", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\p{Emoji_Presentation", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\p{Script=", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\ka", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\k<", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\k<>", ParserOptions::default()),
            (r"\k<>", ParserOptions::default().with_unicode_flags(true, false)),
            (r"\k<a", ParserOptions::default().with_unicode_flags(true, false)),
            ("a(?:", ParserOptions::default()),
            ("(a", ParserOptions::default()),
            ("(?<a>", ParserOptions::default()),
            ("(?=a){1}", ParserOptions::default().with_unicode_flags(true, false)),
            ("(?!a){1}", ParserOptions::default().with_unicode_flags(true, false)),
            (r"[\d-\D]", ParserOptions::default().with_unicode_flags(true, false)),
            ("[z-a]", ParserOptions::default()),
            (r"[a-c]]", ParserOptions::default().with_unicode_flags(true, false)),
            (r"^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$", ParserOptions::default()),
        ] {
            assert!(
                PatternParser::new(&allocator, source_text, *options).parse().is_err(),
                "{source_text} should fail to parse with {options:?}!"
            );
        }
    }

    #[test]
    fn should_handle_unicode() {
        let allocator = Allocator::default();
        let source_text = "このEmoji🥹の数が変わる";

        let pattern = PatternParser::new(
            &allocator,
            source_text,
            ParserOptions::default().with_unicode_flags(true, false),
        )
        .parse()
        .unwrap();
        assert_eq!(pattern.body.body[0].body.len(), 14);
        let pattern = PatternParser::new(
            &allocator,
            source_text,
            ParserOptions::default().with_unicode_flags(true, true),
        )
        .parse()
        .unwrap();
        assert_eq!(pattern.body.body[0].body.len(), 14);

        let pattern =
            PatternParser::new(&allocator, source_text, ParserOptions::default()).parse().unwrap();
        assert_eq!(pattern.body.body[0].body.len(), 15);
    }
}

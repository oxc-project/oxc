use std::fmt::Write;

use itertools::Itertools as _;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_regular_expression::{
    ast::{CapturingGroup, Character, CharacterKind, Pattern},
    visit::{Visit, walk},
};
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::run_on_regex_node};

fn no_control_regex_diagnostic(control_chars: &[Character], span: Span) -> OxcDiagnostic {
    let count = control_chars.len();
    debug_assert!(count > 0);

    let mut octal_chars = Vec::new();
    let mut null_chars = Vec::new();
    let mut other_chars = Vec::new();

    for ch in control_chars {
        match ch.kind {
            CharacterKind::Octal1 | CharacterKind::Octal2 | CharacterKind::Octal3 => {
                octal_chars.push(ch);
            }
            CharacterKind::Null => {
                null_chars.push(ch);
            }
            _ => {
                other_chars.push(ch);
            }
        }
    }

    let mut help = String::new();

    if !other_chars.is_empty() {
        let regexes = other_chars.iter().join(", ");
        writeln!(
            help,
            "'{regexes}' {} {}control character{}.",
            if other_chars.len() > 1 { "are" } else { "is" },
            if other_chars.len() > 1 { "" } else { "a " },
            if other_chars.len() > 1 { "s" } else { "" },
        )
        .unwrap();
    }

    if !octal_chars.is_empty() {
        let regexes = octal_chars.iter().join(", ");
        writeln!(
            help,
            "'{regexes}' {} {}control character{}. They look like backreferences, but there {} no corresponding capture group{}.",
            if octal_chars.len() > 1 { "are" } else { "is" },
            if octal_chars.len() > 1 { "" } else { "a " },
            if octal_chars.len() > 1 { "s" } else { "" },
            if octal_chars.len() > 1 { "are" } else { "is" },
            if octal_chars.len() > 1 { "s" } else { "" }
        ).unwrap();
    }

    if !null_chars.is_empty() {
        writeln!(help, "'\\0' matches the null character (U+0000), which is a control character.")
            .unwrap();
    }

    debug_assert!(!help.is_empty());
    debug_assert!(help.chars().last().is_some_and(|char| char == '\n'));

    if !help.is_empty() {
        help.truncate(help.len() - 1);
    }

    OxcDiagnostic::warn(if count > 1 {
        "Unexpected control characters"
    } else {
        "Unexpected control character"
    })
    .with_help(help)
    .with_label(span)
}
#[derive(Debug, Default, Clone)]
pub struct NoControlRegex;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows control characters and some escape sequences that match
    /// control characters in regular expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Control characters are special, invisible characters in the ASCII range
    /// 0-31. These characters are rarely used in JavaScript strings so a
    /// regular expression containing elements that explicitly match these
    /// characters is most likely a mistake.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var pattern1 = /\x00/;
    /// var pattern2 = /\x0C/;
    /// var pattern3 = /\x1F/;
    /// var pattern4 = /\u000C/;
    /// var pattern5 = /\u{C}/u;
    /// var pattern6 = new RegExp("\x0C"); // raw U+000C character in the pattern
    /// var pattern7 = new RegExp("\\x0C"); // \x0C pattern
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var pattern1 = /\x20/;
    /// var pattern2 = /\u0020/;
    /// var pattern3 = /\u{20}/u;
    /// var pattern4 = /\t/;
    /// var pattern5 = /\n/;
    /// var pattern6 = new RegExp("\x20");
    /// var pattern7 = new RegExp("\\t");
    /// var pattern8 = new RegExp("\\n");
    /// ```
    NoControlRegex,
    eslint,
    correctness
);

impl Rule for NoControlRegex {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        run_on_regex_node(node, ctx, |pattern, span| {
            check_pattern(ctx, pattern, span);
        });
    }
}

fn check_pattern(context: &LintContext, pattern: &Pattern, span: Span) {
    let mut finder = ControlCharacterFinder {
        control_chars: Vec::new(),
        num_capture_groups: 0,
        source_text: context.source_text(),
    };
    finder.visit_pattern(pattern);

    if !finder.control_chars.is_empty() {
        context.diagnostic(no_control_regex_diagnostic(&finder.control_chars, span));
    }
}

#[derive(Default)]
struct ControlCharacterFinder<'a> {
    control_chars: Vec<Character>,
    num_capture_groups: u32,
    source_text: &'a str,
}

impl<'a> Visit<'a> for ControlCharacterFinder<'a> {
    fn visit_pattern(&mut self, it: &Pattern<'a>) {
        walk::walk_pattern(self, it);
        // \1, \2, etc. are sometimes valid "control" characters as they can be
        // used to reference values from capturing groups. Note in this case
        // they're not actually control characters. However, if there's no
        // corresponding capturing group, they _are_ invalid control characters.
        //
        // Some important notes:
        // 1. Capture groups are 1-indexed.
        // 2. Capture groups can be nested.
        // 3. Capture groups may be referenced before they are defined. This is
        //    why we need to do this check here, instead of filtering inside of
        //    visit_character.
        if self.num_capture_groups > 0 && !self.control_chars.is_empty() {
            let control_chars = std::mem::take(&mut self.control_chars);
            let control_chars = control_chars
                .into_iter()
                .filter(|c| !(c.value > 0x01 && c.value <= self.num_capture_groups))
                .collect::<Vec<_>>();
            self.control_chars = control_chars;
        }
    }

    fn visit_character(&mut self, ch: &Character) {
        // Control characters are in the range 0x00 to 0x1F
        if ch.value <= 0x1F {
            let text: &str = ch.span.source_text(self.source_text);
            let is_code_point_match = text
                .trim_start_matches('\\')
                .chars()
                .nth(0)
                .is_some_and(|c| c.to_digit(16) == Some(ch.value));
            if is_code_point_match
                || text.starts_with("\\x")
                || text.starts_with("\\\\x")
                || text.starts_with("\\u")
                || text.starts_with("\\\\u")
            {
                self.control_chars.push(*ch);
            }
        }
    }

    fn visit_capturing_group(&mut self, group: &CapturingGroup<'a>) {
        self.num_capture_groups += 1;
        walk::walk_capturing_group(self, group);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::Tester;

    #[test] //
    fn test_hex_literals() {
        Tester::new(
            NoControlRegex::NAME,
            NoControlRegex::PLUGIN,
            vec![
                "x1f",                 // not a control sequence
                r"new RegExp('\x20')", // control sequence in valid range
                r"new RegExp('\xff')",
                r"let r = /\xff/",
            ],
            vec![r"new RegExp('\x00')", r"/\x00/", r"new RegExp('\x1f')", r"/\x1f/"],
        )
        .test();
    }

    #[test]
    fn test_unicode_literals() {
        Tester::new(
            NoControlRegex::NAME,
            NoControlRegex::PLUGIN,
            vec![
                r"u00",    // not a control sequence
                r"\u00ff", // in valid range
                // multi byte unicode ctl
                r"var re = /^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$/;",
            ],
            vec![
                // regex literal
                r"let r = /\u0000/",
                r"let r = /\u000c/",
                r"let r = /\u000C/",
                r"let r = /\u001f/",
                // invalid utf ctl as literal string
                r"let r = new RegExp('\u0000');",
                r"let r = new RegExp('\u000c');",
                r"let r = new RegExp('\u000C');",
                r"let r = new RegExp('\u001f');",
                // invalid utf ctl pattern
                r"let r = new RegExp('\\u0000');",
                r"let r = new RegExp('\\u000c');",
                r"let r = new RegExp('\\u000C');",
                r"let r = new RegExp('\\u001f');",
            ],
        )
        .test();
    }

    #[test]
    fn test_unicode_brackets() {
        Tester::new(
            NoControlRegex::NAME,
            NoControlRegex::PLUGIN,
            vec![
                r"let r = /\u{0}/", // no unicode flag, this is valid
                r"let r = /\u{ff}/u",
                r"let r = /\u{00ff}/u",
                r"let r = new RegExp('\\u{1F}', flags);", // flags are unknown
            ],
            vec![
                r"let r = /\u{0}/u",
                r"let r = new RegExp('\\u{0}', 'u');",
                r"let r = /\u{c}/u",
                r"let r = /\u{1F}/u",
                r"let r = new RegExp('\\u{1F}', 'u');", // flags are known & contain u
            ],
        )
        .test();
    }

    #[test]
    fn test_capture_group_indexing() {
        // https://github.com/oxc-project/oxc/issues/6525
        let pass = vec![
            r#"const filename = /filename[^;=\n]=((['"]).?\2|[^;\n]*)/;"#,
            r"const r = /([a-z])\1/;",
            r"const r = /\1([a-z])/;",
        ];
        let fail = vec![
            r"const r = /\0/;",
            r"const r = /[a-z]\1/;",
            r"const r = /([a-z])\2/;",
            r"const r = /([a-z])\0/;",
        ];
        Tester::new(NoControlRegex::NAME, NoControlRegex::PLUGIN, pass, fail)
            .with_snapshot_suffix("capture-group-indexing")
            .test_and_snapshot();
    }

    #[test]
    fn test() {
        // test cases taken from eslint. See:
        // https://github.com/eslint/eslint/blob/v9.9.1/tests/lib/rules/no-control-regex.js
        Tester::new(
            NoControlRegex::NAME,
            NoControlRegex::PLUGIN,
            vec![
                "var regex = /x1f/;",
                r"var regex = /\\x1f/",
                "var regex = new RegExp(\"x1f\");",
                "var regex = RegExp(\"x1f\");",
                "new RegExp('[')",
                "RegExp('[')",
                "new (function foo(){})('\\x1f')",
                r"/\u{20}/u",
                r"/\u{1F}/",
                r"/\u{1F}/g",
                r"new RegExp('\\u{20}', 'u')",
                r"new RegExp('\\u{20}', `u`)",
                r"new RegExp('\\u{1F}')",
                r"new RegExp(`\\u{1F}`)",
                r"new RegExp('\\u{1F}', 'g')",
                r"new RegExp('\\u{1F}', flags)", // unknown flags, we assume no 'u'
                // https://github.com/oxc-project/oxc/issues/6136
                r"/---\n([\s\S]+?)\n---/",
                r"/import \{((?:.|\n)*)\} from '@romejs\/js-ast';/",
                r"/^\t+/",
                r"/\n/g",
                r"/\r\n|\r|\n/",
                r"/[\n\r\p{Z}\p{P}]/u",
                r"/[\n\t]+/g",
                r"/^expected `string`\.\n {2}in Foo \(at (.*)[/\\]debug[/\\]test[/\\]browser[/\\]debug\.test\.js:[0-9]+\)$/",
                r"/\f/",
                r"/\v/",
            ],
            vec![
                r"var regex = /\x1f/",
                r"var regex = /\\\x1f\\x1e/",
                r"var regex = /\\\x1fFOO\\x00/",
                r"var regex = /FOO\\\x1fFOO\\x1f/",
                "var regex = new RegExp('\\x1f\\x1e')",
                "var regex = new RegExp('\\x1fFOO\\x00')",
                "var regex = new RegExp('FOO\\x1fFOO\\x1f')",
                "var regex = RegExp('\\x1f')",
                "var regex = /(?<a>\\x1f)/",
                r"var regex = /(?<\u{1d49c}>.)\x1f/",
                r"new RegExp('\\u{1111}*\\x1F', 'u')",
                r"/\u{1F}/u",
                r"/\u{1F}/ugi",
                r"new RegExp('\\u{1F}', 'u')",
                r"new RegExp(`\\u{1F}`, 'u')",
                r"new RegExp('\\u{1F}', `u`)",
                r"new RegExp('\\u{1F}', 'ugi')",
                // https://github.com/oxc-project/oxc/issues/6136
                r"/\u{0a}/u",
                r"/\x0a/u",
                r"/\u{0d}/u",
                r"/\x0d/u",
                r"/\u{09}/u",
                r"/\x09/u",
                r"/\0\1\2/",
                r"/\x1f\2/",
                r"/\x1f\0/",
                r"/\x1f\0\2/",
                // globalThis
                "var regex = new globalThis.RegExp('\\x1f\\x1e')",
                "var regex = globalThis.RegExp('\\x1f')",
                // inner expressions (parentheses and type expressions)
                "RegExp(('\\x1f'))",
                "new RegExp(('\\x1f'))",
                "new RegExp((('\\x1f')))",
                "new RegExp('\\x1f' as string)",
            ],
        )
        .test_and_snapshot();
    }
}

use lazy_static::lazy_static;
use oxc_ast::{
    ast::{Argument, Expression, RegExpFlags},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, GetSpan, Span};
use regex::{Matches, Regex};

use crate::{ast_util::extract_regex_flags, context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-control-regex): Unexpected control character(s)")]
#[diagnostic(
    severity(warning),
    help("Unexpected control character(s) in regular expression: \"{0}\"")
)]
struct NoControlRegexDiagnostic(Atom, #[label] pub Span);

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
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    ///
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
    ///
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
    correctness
);

impl Rule for NoControlRegex {
    fn run<'a>(&self, node: &AstNode<'a>, context: &LintContext<'a>) {
        if let Some(RegexPatternData { pattern, flags, span }) = regex_pattern(node) {
            let mut violations: Vec<&str> = Vec::new();

            for matched_ctl_pattern in control_patterns(pattern) {
                let ctl = matched_ctl_pattern.as_str();

                // check for an even number of backslashes, since these will
                // prevent the pattern from being a control sequence
                if ctl.starts_with('\\') && matched_ctl_pattern.start() > 0 {
                    let pattern_chars: Vec<char> = pattern.chars().collect(); // ew

                    // Convert byte index to char index
                    let byte_start = matched_ctl_pattern.start();
                    let char_start = pattern[..byte_start].chars().count();

                    let mut first_backslash = char_start;
                    while first_backslash > 0 && pattern_chars[first_backslash] == '\\' {
                        first_backslash -= 1;
                    }

                    let mut num_slashes = char_start - first_backslash;
                    if first_backslash == 0 && pattern_chars[first_backslash] == '\\' {
                        num_slashes += 1;
                    }
                    // even # of slashes
                    if num_slashes % 2 == 0 {
                        continue;
                    }
                }

                if ctl.starts_with(r"\x") || ctl.starts_with(r"\u") {
                    let mut numeric_part = &ctl[2..];

                    // extract numeric part from \u{00}
                    if numeric_part.starts_with('{') {
                        let has_unicode_flag = match flags {
                            Some(flags) if flags.contains(RegExpFlags::U) => true,
                            _ => {
                                continue;
                            }
                        };

                        // 1. Unicode control pattern is missing a curly brace
                        //    and is therefore invalid. (note: we may want to
                        //    report this?)
                        // 2. Unicode flag is missing, which is needed for
                        //    interpreting \u{`nn`} as a unicode character
                        if !has_unicode_flag || !numeric_part.ends_with('}') {
                            continue;
                        }

                        numeric_part = &numeric_part[1..numeric_part.len() - 1];
                    }

                    match u32::from_str_radix(numeric_part, 16) {
                        Err(_) => continue,
                        Ok(as_num) if as_num > 0x1f => continue,
                        Ok(_) => { /* noop */ }
                    }
                }

                violations.push(ctl);
            }

            if !violations.is_empty() {
                let violations = violations.join(", ");
                context.diagnostic(NoControlRegexDiagnostic(violations.into(), span));
            }
        }
    }
}

struct RegexPatternData<'a> {
    /// A regex pattern, either from a literal (`/foo/`) a RegExp constructor
    /// (`new RegExp("foo")`), or a RegExp function call (`RegExp("foo"))
    pattern: &'a Atom,
    /// Regex flags, if found. It's possible for this to be `Some` but have
    /// no flags.
    ///
    /// Note that flags are represented by a `u8` and therefore safely clonable
    /// with low performance overhead.
    flags: Option<RegExpFlags>,
    /// The pattern's span. For [`Expression::NewExpression`]s and [`Expression::CallExpression`]s,
    /// this will match the entire new/call expression.
    ///
    /// Note that spans are 8 bytes and safely clonable with low performance overhead
    span: Span,
}

/// Returns the regex pattern inside a node, if it's applicable.
///
/// e.g.:
/// * /foo/ -> "foo"
/// * new RegExp("foo") -> foo
///
/// note: [`RegExpFlags`] and [`Span`]s are both tiny and cloneable.
fn regex_pattern<'a>(node: &AstNode<'a>) -> Option<RegexPatternData<'a>> {
    let kind = node.kind();
    match kind {
        // regex literal
        AstKind::RegExpLiteral(reg) => Some(RegexPatternData {
            pattern: &reg.regex.pattern,
            flags: Some(reg.regex.flags),
            span: reg.span,
        }),

        // new RegExp()
        AstKind::NewExpression(expr) => {
            // constructor is RegExp,
            if expr.callee.is_specific_id("RegExp")
            // which is provided at least 1 parameter,
                && expr.arguments.len() > 0
            {
                // where the first one is a string literal
                // note: improvements required for strings used via identifier
                // references
                if let Argument::Expression(Expression::StringLiteral(pattern)) = &expr.arguments[0]
                {
                    // get pattern from arguments. Missing or non-string arguments
                    // will be runtime errors, but are not covered by this rule.
                    // Note that we're intentionally reporting the entire "new
                    // RegExp("pat") expression, not just "pat".
                    Some(RegexPatternData {
                        pattern: &pattern.value,
                        flags: extract_regex_flags(&expr.arguments),
                        span: kind.span(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }

        // RegExp()
        AstKind::CallExpression(expr) => {
            // constructor is RegExp,
            if expr.callee.is_specific_id("RegExp")
                // which is provided at least 1 parameter,
                && expr.arguments.len() > 0
            {
                // where the first one is a string literal
                // note: improvements required for strings used via identifier
                // references
                if let Argument::Expression(Expression::StringLiteral(pattern)) = &expr.arguments[0]
                {
                    // get pattern from arguments. Missing or non-string arguments
                    // will be runtime errors, but are not covered by this rule.
                    // Note that we're intentionally reporting the entire "new
                    // RegExp("pat") expression, not just "pat".
                    Some(RegexPatternData {
                        pattern: &pattern.value,
                        flags: extract_regex_flags(&expr.arguments),
                        span: kind.span(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn control_patterns(pattern: &Atom) -> Matches<'static, '_> {
    lazy_static! {
        static ref CTL_PAT: Regex = Regex::new(
            r"([\x00-\x1f]|(?:\\x\w{2})|(?:\\u\w{4})|(?:\\u\{\w{1,4}\}))"
            // r"((?:\\x\w{2}))"
        ).unwrap();
    }
    CTL_PAT.find_iter(pattern.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tester::Tester;

    #[test]
    fn test_hex_literals() {
        Tester::new_without_config(
            NoControlRegex::NAME,
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
        Tester::new_without_config(
            NoControlRegex::NAME,
            vec![
                r"u00",    // not a control sequence
                r"\u00ff", // in valid range
                // multi byte unicode ctl
                r"var re = /^([a-zªµºß-öø-ÿāăąćĉċčďđēĕėęěĝğġģĥħĩīĭįıĳĵķ-ĸĺļľŀłńņň-ŉŋōŏőœŕŗřśŝşšţťŧũūŭůűųŵŷźżž-ƀƃƅƈƌ-ƍƒƕƙ-ƛƞơƣƥƨƪ-ƫƭưƴƶƹ-ƺƽ-ƿǆǉǌǎǐǒǔǖǘǚǜ-ǝǟǡǣǥǧǩǫǭǯ-ǰǳǵǹǻǽǿȁȃȅȇȉȋȍȏȑȓȕȗșțȝȟȡȣȥȧȩȫȭȯȱȳ-ȹȼȿ-ɀɂɇɉɋɍɏ-ʓʕ-ʯͱͳͷͻ-ͽΐά-ώϐ-ϑϕ-ϗϙϛϝϟϡϣϥϧϩϫϭϯ-ϳϵϸϻ-ϼа-џѡѣѥѧѩѫѭѯѱѳѵѷѹѻѽѿҁҋҍҏґғҕҗҙқҝҟҡңҥҧҩҫҭүұҳҵҷҹһҽҿӂӄӆӈӊӌӎ-ӏӑӓӕӗәӛӝӟӡӣӥӧөӫӭӯӱӳӵӷӹӻӽӿԁԃԅԇԉԋԍԏԑԓԕԗԙԛԝԟԡԣա-ևᴀ-ᴫᵢ-ᵷᵹ-ᶚḁḃḅḇḉḋḍḏḑḓḕḗḙḛḝḟḡḣḥḧḩḫḭḯḱḳḵḷḹḻḽḿṁṃṅṇṉṋṍṏṑṓṕṗṙṛṝṟṡṣṥṧṩṫṭṯṱṳṵṷṹṻṽṿẁẃẅẇẉẋẍẏẑẓẕ-ẝẟạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỳỵỷỹỻỽỿ-ἇἐ-ἕἠ-ἧἰ-ἷὀ-ὅὐ-ὗὠ-ὧὰ-ώᾀ-ᾇᾐ-ᾗᾠ-ᾧᾰ-ᾴᾶ-ᾷιῂ-ῄῆ-ῇῐ-ΐῖ-ῗῠ-ῧῲ-ῴῶ-ῷⁱⁿℊℎ-ℏℓℯℴℹℼ-ℽⅆ-ⅉⅎↄⰰ-ⱞⱡⱥ-ⱦⱨⱪⱬⱱⱳ-ⱴⱶ-ⱼⲁⲃⲅⲇⲉⲋⲍⲏⲑⲓⲕⲗⲙⲛⲝⲟⲡⲣⲥⲧⲩⲫⲭⲯⲱⲳⲵⲷⲹⲻⲽⲿⳁⳃⳅⳇⳉⳋⳍⳏⳑⳓⳕⳗⳙⳛⳝⳟⳡⳣ-ⳤⴀ-ⴥꙁꙃꙅꙇꙉꙋꙍꙏꙑꙓꙕꙗꙙꙛꙝꙟꙣꙥꙧꙩꙫꙭꚁꚃꚅꚇꚉꚋꚍꚏꚑꚓꚕꚗꜣꜥꜧꜩꜫꜭꜯ-ꜱꜳꜵꜷꜹꜻꜽꜿꝁꝃꝅꝇꝉꝋꝍꝏꝑꝓꝕꝗꝙꝛꝝꝟꝡꝣꝥꝧꝩꝫꝭꝯꝱ-ꝸꝺꝼꝿꞁꞃꞅꞇꞌﬀ-ﬆﬓ-ﬗａ-ｚ]|\ud801[\udc28-\udc4f]|\ud835[\udc1a-\udc33\udc4e-\udc54\udc56-\udc67\udc82-\udc9b\udcb6-\udcb9\udcbb\udcbd-\udcc3\udcc5-\udccf\udcea-\udd03\udd1e-\udd37\udd52-\udd6b\udd86-\udd9f\uddba-\uddd3\uddee-\ude07\ude22-\ude3b\ude56-\ude6f\ude8a-\udea5\udec2-\udeda\udedc-\udee1\udefc-\udf14\udf16-\udf1b\udf36-\udf4e\udf50-\udf55\udf70-\udf88\udf8a-\udf8f\udfaa-\udfc2\udfc4-\udfc9\udfcb])$/;",
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
        Tester::new_without_config(
            NoControlRegex::NAME,
            vec![
                r"let r = /\u{0}/", // no unicode flag, this is valid
                r"let r = /\u{ff}/u",
                r"let r = /\u{00ff}/u",
                r"let r = new RegExp('\\u{1F}', flags);", // flags are unknown
            ],
            vec![
                r"let r = /\u{0}/u",
                r"let r = /\u{c}/u",
                r"let r = /\u{1F}/u",
                r"let r = new RegExp('\\u{1F}', 'u');", // flags are known & contain u
            ],
        )
        .test();
    }

    #[test]
    fn test() {
        // test cases taken from eslint. See:
        // https://github.com/eslint/eslint/blob/main/tests/lib/rules/no-control-regex.js
        Tester::new_without_config(
            NoControlRegex::NAME,
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
                r"new RegExp('\\u{1F}')",
                r"new RegExp('\\u{1F}', 'g')",
                r"new RegExp('\\u{1F}', flags)", // unknown flags, we assume no 'u'
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
                r"new RegExp('\\u{1F}', 'ugi')",
            ],
        )
        .test_and_snapshot();
    }
}

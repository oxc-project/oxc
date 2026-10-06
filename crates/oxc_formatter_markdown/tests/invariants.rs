//! Fuzzed safety invariants (the parser repo's differential corpus, formatted instead of compared):
//!
//! 1. `parse(format(x)) ≅ parse(x)` (the fixture fingerprint)
//! 2. `format(format(x)) == format(x)`
//!
//! under every `proseWrap`.
//! Deterministic; the default corpus must fail exactly on the known documents (`KNOWN_FAILURES`),
//! any other corpus reports every failure:
//! `MD_FUZZ_COUNT=50000 MD_FUZZ_SEED=7 cargo test -p oxc_formatter_markdown --test invariants`.
//! `MD_FUZZ_FILE=path` prints one document's fingerprints.

#![expect(clippy::print_stderr)] // the failure reports

use oxc_allocator::Allocator;
use oxc_formatter_core::{IndentWidth, LineWidth};
use oxc_formatter_markdown::{MarkdownFormatOptions, ProseWrap, format, parse_for_format};

#[path = "fixtures/fingerprint.rs"]
mod fingerprint;

/// The differential runner's token soup (`tasks/differential/run.ts` in the parser repo).
const TOKENS: &[&str] = &[
    "*",
    "**",
    "_",
    "__",
    "`",
    "``",
    "[",
    "]",
    "(",
    ")",
    "<",
    ">",
    "#",
    "##",
    "-",
    "+",
    "1.",
    "2)",
    ">",
    "\\",
    "&",
    "&amp;",
    "&#35;",
    "!",
    "\"",
    "'",
    "=",
    "~",
    "|",
    ":",
    "a",
    "foo",
    "bar b",
    "é",
    "漢字",
    "http://x.y",
    "user@e.com",
    "\\*",
    "\\\\",
    " ",
    "  ",
    "\t",
    "\n",
    "\n\n",
    "  \n",
    "\\\n",
    "```\n",
    "---",
    "===",
    "***",
    "    ",
    "[x]: /u",
    "[x]",
    "\"t\"",
    "'t'",
    "<div>",
    "</div>",
    "<!-- c -->",
    "![",
    "](u)",
    "<a>",
    "<span x=1>",
    "</span>",
    "> x\n<a>\n",
    "<script>",
    "</script>",
    "<pre x>",
    "</pre>",
    "<textarea>\n",
    "<style",
    "</style>",
    "~~",
    "~",
    "[^1]",
    "[^1]: n",
    "- [x] ",
    "- [ ] ",
    "| a | b |",
    "| - | - |",
    "-|-",
    ":-:",
    "www.a.com",
    "http://x.y/p(q)",
    "u@e.com",
    "W",
    "h",
    "[σ]: /u",
    "[ς]",
    "[ﬀ]: /u",
    "[ff]",
    "\u{3000}",
    "\u{2E53}",
    "\u{FDFE}",
    "😀",
    "$",
    "$$",
    "$$\n",
    "$$x$$",
    "$$m\n",
    "{%",
    "%}",
    "{{",
    "}}",
    "{{ v }}",
    "{% t %}",
    "[[",
    "]]",
    "[[w]]",
    ":::",
    "::::",
    ":::\n",
    ":::note\n",
    ":::note",
    "::::a\n",
    "[l]",
    "{.c}",
    ":::a[l]{#i k=v}\n",
    "::: tip Title\n",
    "    a\n",
    "    \n",
    "\t\n",
    "      \n",
    "  \t\r\n",
    "\r\n",
    "  \r\n",
    // Prose shapes the formatter wraps: CJK runs, long words, alerts, component tags
    "日本語の文章",
    "한국어 단어",
    "、",
    "。",
    "「",
    "」",
    "…",
    "〜",
    "English words",
    "[!NOTE]",
    "<Badge />",
    "<<< @/f.js",
    "(paren)",
    "a * b",
    "2. ",
];

// The default corpus (seed 0, 3000 documents) fails exactly on these documents.
// Any failure is a regression; a new known class goes here with its description.
// Other seeds are close to 0 too; what remains there is a lazy line inside a liquid tag
// (re-indenting the continuation turns the tag into a flow block).
const KNOWN_FAILURES: &[(usize, ProseWrap)] = &[];

/// The runner's `Rng` (mulberry32), so a seed names the same document on both sides.
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 0x3d));
        t ^ (t >> 14)
    }

    /// The runner's `Math.floor(random() * (hi - lo + 1)) + lo`, in integers (the product fits).
    fn int(&mut self, lo: usize, hi: usize) -> usize {
        let n = (u64::from(self.next()) * (hi - lo + 1) as u64) >> 32;
        lo + usize::try_from(n).unwrap()
    }

    fn soup(&mut self, n: usize) -> String {
        std::iter::repeat_with(|| TOKENS[self.int(0, TOKENS.len() - 1)]).take(n).collect()
    }
}

fn format_source(source: &str, options: MarkdownFormatOptions) -> Option<String> {
    let allocator = Allocator::default();
    Some(format(&allocator, source, options).ok()?.print().ok()?.into_code())
}

fn fingerprint_of(source: &str) -> String {
    let allocator = Allocator::default();
    let parsed = parse_for_format(&allocator, source).expect("source should parse");
    fingerprint::fingerprint(parsed.source, parsed.root)
}

fn first_difference<'a>(before: &'a str, after: &'a str) -> (&'a str, &'a str) {
    let mut a = after.lines();
    for b in before.lines() {
        match a.next() {
            Some(l) if l == b => {}
            Some(l) => return (b, l),
            None => return (b, "<end>"),
        }
    }
    ("<end>", a.next().unwrap_or("<end>"))
}

/// `MD_FUZZ_FILE=path`: the invariants of one document, with the full fingerprints
/// (`MD_FUZZ_WIDTH` / `MD_FUZZ_TAB_WIDTH` override the corpus options).
#[test]
fn single_file() {
    let Ok(path) = std::env::var("MD_FUZZ_FILE") else { return };
    let source = std::fs::read_to_string(path).unwrap();
    let env = |name: &str, default: u16| -> u16 {
        std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    };
    let line_width = LineWidth::try_from(env("MD_FUZZ_WIDTH", 40)).unwrap();
    let indent_width =
        IndentWidth::try_from(u8::try_from(env("MD_FUZZ_TAB_WIDTH", 2)).unwrap()).unwrap();
    for prose_wrap in [ProseWrap::Preserve, ProseWrap::Always, ProseWrap::Never] {
        let options = MarkdownFormatOptions {
            prose_wrap,
            line_width,
            indent_width,
            ..MarkdownFormatOptions::default()
        };
        let once = format_source(&source, options).unwrap();
        let twice = format_source(&once, options).unwrap();
        eprintln!(
            "=== {prose_wrap:?}\n--- once\n{once}--- before\n{}--- after\n{}--- idempotent: {}",
            fingerprint_of(&source),
            fingerprint_of(&once),
            once == twice
        );
    }
}

#[test]
fn fuzzed_invariants() {
    let count: usize =
        std::env::var("MD_FUZZ_COUNT").ok().and_then(|v| v.parse().ok()).unwrap_or(3000);
    let seed: u32 = std::env::var("MD_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let mut rng = Rng(seed);
    let mut failures: Vec<((usize, ProseWrap), String)> = Vec::new();
    for i in 0..count {
        let n = rng.int(2, 40);
        let source = rng.soup(n).trim_end_matches('\n').to_string() + "\n";
        for prose_wrap in [ProseWrap::Preserve, ProseWrap::Always, ProseWrap::Never] {
            let options = MarkdownFormatOptions {
                prose_wrap,
                line_width: LineWidth::try_from(40).unwrap(),
                ..MarkdownFormatOptions::default()
            };
            let Some(once) = format_source(&source, options) else {
                failures.push(((i, prose_wrap), format!("format failed\n{source:?}")));
                continue;
            };
            let before = fingerprint_of(&source);
            let after = fingerprint_of(&once);
            if before != after {
                let (b, a) = first_difference(&before, &after);
                failures.push((
                    (i, prose_wrap),
                    format!("meaning changed\ninput: {source:?}\noutput: {once:?}\nbefore: {b}\nafter:  {a}"),
                ));
                continue;
            }
            let twice = format_source(&once, options).unwrap_or_default();
            if twice != once {
                failures.push((
                    (i, prose_wrap),
                    format!("not idempotent\ninput: {source:?}\nonce: {once:?}\ntwice: {twice:?}"),
                ));
            }
        }
    }
    let known: &[(usize, ProseWrap)] =
        if count == 3000 && seed == 0 { KNOWN_FAILURES } else { &[] };
    let failed: Vec<_> = failures.iter().map(|(key, _)| *key).collect();
    if failed != known {
        let report: Vec<_> = failures
            .iter()
            .take(60)
            .map(|((i, prose_wrap), what)| format!("#{i} {prose_wrap:?}: {what}"))
            .collect();
        eprintln!(
            "{} of {count} documents (seed {seed}) broke an invariant; first 60:\n\n{}",
            failures.len(),
            report.join("\n\n")
        );
    }
    assert_eq!(failed, known, "failures on seed {seed} differ from the known ones");
}

use lazy_regex::Regex;
use oxc_ast::{AstKind, ast::JSXAttributeName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::deserialize_regex_vec,
};

fn prefer_https_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `https://` over `http://`.")
        .with_help("Change the URL to use the `https://` protocol.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferHttps(Box<PreferHttpsConfig>);

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PreferHttpsConfig {
    // exact-match strings or anchored regexes for partial matches
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    ignore: Vec<Regex>,
}

impl std::ops::Deref for PreferHttps {
    type Target = PreferHttpsConfig;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `https://` protocol over `http://`.
    ///
    /// ### Why is this bad?
    ///
    /// Using `http://` is insecure. Prefer `https://` wherever possible.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const url = "http://example.com";
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const url = "https://example.com";
    /// ```
    PreferHttps,
    unicorn,
    nursery,
    fix,
    config = PreferHttpsConfig,
    version = "next",
    short_description = "Prefer `https://` over `http://`.",
);

impl Rule for PreferHttps {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<PreferHttpsConfig>::from_value(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::StringLiteral(string_lit) => {
                let value = string_lit.value.as_str();

                if !value.starts_with("http://") {
                    return;
                }

                if is_xmlns_attribute_value(node, ctx) {
                    return;
                }

                if !self.is_violation(value) {
                    return;
                }

                let http_scan = Span::sized(string_lit.span.start + 1, 4);
                ctx.diagnostic_with_fix(prefer_https_diagnostic(string_lit.span), |fixer| {
                    fixer.replace(http_scan, "https")
                });
            }

            AstKind::TemplateLiteral(tpl) if tpl.expressions.is_empty() => {
                if let Some(quasi) = tpl.quasis.first()
                    && let Some(raw) = quasi.value.cooked.as_ref()
                {
                    let value = raw.as_str();
                    if value.starts_with("http://") {
                        self.report_if_violation(value, tpl.span, ctx);
                    }
                }
            }

            AstKind::JSXText(text) => {
                let value = text.value.as_str();
                if value.contains("http://") {
                    self.check_substrings(value, text.span, ctx);
                }
            }
            _ => {}
        }
    }
    fn run_once(&self, ctx: &LintContext<'_>) {
        for comment in ctx.comments() {
            let text = comment.span.source_text(ctx.source_text());
            if text.contains("http://") {
                self.check_substrings(text, comment.span, ctx);
            }
        }
    }
}

impl PreferHttps {
    fn check_substrings(&self, text: &str, base_span: Span, ctx: &LintContext<'_>) {
        let mut search_start = 0usize;
        while let Some(rel_idx) = text[search_start..].find("http://") {
            let start = search_start + rel_idx;
            let rest = &text[start..];
            let end = rest
                .find(|c: char| {
                    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | '`' | ',')
                })
                .unwrap_or(rest.len());
            let url = &rest[..end];
            let abs_start = base_span.start + u32::try_from(start).unwrap_or(u32::MAX);
            let abs_end = abs_start + u32::try_from(url.len()).unwrap_or(u32::MAX);
            self.report_if_violation(url, Span::new(abs_start, abs_end), ctx);
            search_start = start + url.len().max(1);
        }
    }

    fn is_violation(&self, value: &str) -> bool {
        if is_local_or_no_dot_host(value) {
            return false;
        }
        if KNOWN_SAFE_URLS.contains(&value) {
            return false;
        }
        if self.ignore.iter().any(|pattern| ignore_matches(pattern, value)) {
            return false;
        }
        true
    }

    fn report_if_violation(&self, value: &str, span: Span, ctx: &LintContext<'_>) {
        if self.is_violation(value) {
            ctx.diagnostic(prefer_https_diagnostic(span));
        }
    }
}

fn ignore_matches(pattern: &Regex, value: &str) -> bool {
    let src = pattern.as_str();
    let has_explicit_anchor = src.contains('^') || src.contains('$');

    if has_explicit_anchor {
        pattern.is_match(value)
    } else {
        match pattern.find(value) {
            Some(m) => m.start() == 0 && m.end() == value.len(),
            None => false,
        }
    }
}

fn is_xmlns_attribute_value<'a>(node: &AstNode<'a>, ctx: &LintContext<'a>) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let AstKind::JSXAttribute(attr) = parent.kind() else {
        return false;
    };

    match &attr.name {
        JSXAttributeName::Identifier(ident) => ident.name == "xmlns",
        JSXAttributeName::NamespacedName(ns) => ns.namespace.name == "xmlns",
    }
}

fn is_local_or_no_dot_host(url: &str) -> bool {
    let after_scheme = &url["http://".len()..];
    let authority_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    let authority = &after_scheme[..authority_end];

    let host_and_port = match authority.rfind('@') {
        Some(at_pos) => &authority[at_pos + 1..],
        None => authority,
    };

    if host_and_port.starts_with('[') {
        return true;
    }

    let host = match host_and_port.find(':') {
        Some(colon_pos) => {
            let port = &host_and_port[colon_pos + 1..];
            if !port.is_empty() && !port.bytes().all(|b| b.is_ascii_digit()) {
                return true;
            }
            &host_and_port[..colon_pos]
        }
        None => host_and_port,
    };

    if host == "localhost" || host == "127.0.0.1" || host.starts_with('[') || !host.contains('.') {
        return true;
    }

    let Some(last_dot) = host.rfind('.') else {
        return true;
    };

    let tld = &host[last_dot + 1..];
    !tld.is_empty() && tld.chars().all(|c| c.is_ascii_digit())
}

const KNOWN_SAFE_URLS: &[&str] = &[
    "http://www.w3.org/2000/svg",
    "http://www.w3.org/1999/xhtml",
    "http://www.w3.org/1999/XSL/Format",
    "http://www.w3.org/2000/xmlns/",
    "http://www.w3.org/1998/Math/MathML",
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
    "http://schemas.xmlsoap.org/soap/envelope/",
    "http://purl.org/dc/elements/1.1/",
    "http://www.sitemaps.org/schemas/sitemap/0.9",
    "http://www.w3.org/2009/xmldsig11#future-algorithm",
    "http://www.w3.org/2009/xmlenc11#future-algorithm",
    "http://www.w3.org/2099/12/xmldsig-filter3",
    "http://www.w3.org/2099/12/xml-c14n12#WithComments",
];

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"const url = "https://sindresorhus.com";"#, None),
        (r#"const url = "https://example.com/http://sindresorhus.com";"#, None),
        (r#"const url = "HTTP://sindresorhus.com";"#, None),
        (r#"const url = "HTTP://example.com/http://sindresorhus.com";"#, None),
        (r#"const url = "http://localhost";"#, None),
        (r#"const url = "http://example";"#, None),
        (r#"const url = "http://127.0.0.1";"#, None),
        (r#"const url = "http://[::1]";"#, None),
        (r#"const url = "http://example.123";"#, None),
        (r#"const url = "http://sindresorhus.com:invalid";"#, None),
        (r#"const text = "prefixhttp://sindresorhus.com";"#, None),
        ("// http://example,", None),
        ("// http://localhost", None),
        (
            r#"const element = <a href="https://sindresorhus.com">https://sindresorhus.com</a>;"#,
            None,
        ),
        (
            "// eslint-disable-next-line rule-to-test/prefer-https
                    // http://sindresorhus.com",
            None,
        ),
        (r#"const element = <svg xmlns="http://www.w3.org/2000/svg"></svg>;"#, None),
        (
            r#"const element = <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d=""/></svg>;"#,
            None,
        ),
        (r#"const element = <html xmlns="http://www.w3.org/1999/xhtml"></html>;"#, None),
        (r#"const element = <tag xmlns:ns="http://example.com/ns"></tag>;"#, None),
        (r#"const element = <tag xmlns:xsl-fo="http://www.w3.org/1999/XSL/Format"></tag>;"#, None),
        ("const element = <svg xmlns='http://www.w3.org/2000/svg'></svg>;", None),
        (r#"const element = <svg xmlns = "http://www.w3.org/2000/svg"></svg>;"#, None),
        (r#"const SVG_NAMESPACE = "http://www.w3.org/2000/svg";"#, None),
        (r#"document.createElementNS("http://www.w3.org/2000/svg", "svg");"#, None),
        (r#"svg.setAttributeNS("http://www.w3.org/2000/xmlns/", "xmlns", SVG_NAMESPACE);"#, None),
        (r#"const ns = "http://www.w3.org/1998/Math/MathML";"#, None),
        (r#"const ns = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";"#, None),
        (r#"const ns = "http://schemas.xmlsoap.org/soap/envelope/";"#, None),
        (r#"const ns = "http://purl.org/dc/elements/1.1/";"#, None),
        (r#"const ns = "http://www.sitemaps.org/schemas/sitemap/0.9";"#, None),
        (r#"const uri = "http://www.w3.org/2009/xmldsig11#future-algorithm";"#, None),
        (r#"const uri = "http://www.w3.org/2009/xmlenc11#future-algorithm";"#, None),
        (r#"const uri = "http://www.w3.org/2099/12/xmldsig-filter3";"#, None),
        (r#"const uri = "http://www.w3.org/2099/12/xml-c14n12#WithComments";"#, None),
        (
            r#"const uri = "http://example.com/identifier/value";"#,
            Some(serde_json::json!([{"ignore": ["http://example.com/identifier/value"]}])),
        ),
        (
            r#"const url = "http://example.com/http://sindresorhus.com";"#,
            Some(serde_json::json!([{"ignore": ["http://example.com/http://sindresorhus.com"]}])),
        ),
        (
            r#"const uri = "http://schemas.example.com/value";"#,
            Some(serde_json::json!([{"ignore": [r"^http://schemas\.example\.com/"]}])),
        ),
        (
            r#"const uris = ["http://example.com/one", "http://example.com/two"];"#,
            Some(
                serde_json::json!([{"ignore": [r"^http://unused\.example/", r"^http://example\.com/"]}]),
            ),
        ),
        (
            r#"const url = "http://example.com/identifier/value?format=xml#section";"#,
            Some(serde_json::json!([{"ignore": [r"\?format=xml#section$"]}])),
        ),
    ];

    let fail = vec![
        (r#"const url = "http://sindresorhus.com";"#, None),
        ("const url = `http://sindresorhus.com/path`;", None),
        ("// http://sindresorhus.com", None),
        ("// http://sindresorhus.com,", None),
        (
            r#"const element = <a href="http://sindresorhus.com">https://sindresorhus.com</a>;"#,
            None,
        ),
        (
            r#"const element = <a href="https://sindresorhus.com">http://sindresorhus.com</a>;"#,
            None,
        ),
        (
            r#"const urls = [
                        "http://sindresorhus.com",
                        "http://example.com",
                    ];"#,
            None,
        ),
        (r#"const url = "http://user:password@sindresorhus.com";"#, None),
        (r#"const url = "http://sindresorhus.com/path@localhost";"#, None),
        (r#"const url = "http://êxample.com";"#, None),
        (r#"const url = "http://sindresorhus.com:8080/path";"#, None),
        (r#"const url = "http://example.com/http://sindresorhus.com";"#, None),
        (r#"const url = "http://sindresorhus.com.";"#, None),
        (
            r#"const element = <svg xmlns="http://www.w3.org/2000/svg" data-url="http://sindresorhus.com"></svg>;"#,
            None,
        ),
        (r#"const element = <div data-xmlns="http://sindresorhus.com"></div>;"#, None),
        (r#"const $xmlns = "http://sindresorhus.com";"#, None),
        (r#"const url = "http://www.w3.org/2000/svg/extra/path";"#, None),
        (r#"const url = "http://www.w3.org";"#, None),
        (
            r#"const uri = "http://example.com/identifier/value";"#,
            Some(serde_json::json!([{"ignore": ["http://other.example/identifier/value"]}])),
        ),
        (
            r#"const uri = "http://example.com/identifier/value/extra";"#,
            Some(serde_json::json!([{"ignore": ["http://example.com/identifier/value"]}])),
        ),
        (r#"const uri = "http://www.w3.org/2000/09/xmldsig-other#sha1";"#, None),
        (r#"const uri = "http://www.w3.org/2000/09/xmldsig/path";"#, None),
        (r#"const uri = "http://www.w3.org/2002/06/xmldsig-filter2/extra";"#, None),
        (r#"const uri = "http://www.w3.org/2006/12/xml-c14n11?format=xml";"#, None),
        (r#"const uri = "http://www.w3.org/2005/xpath-functions/map/extra";"#, None),
        (r#"const uri = "http://www.w3.org/2001/XInclude/extra";"#, None),
        (r#"const uri = "http://schemas.android.com/apk/res/android/extra";"#, None),
        (r#"const uri = "http://schemas.android.com/apk/res/android?theme=dark";"#, None),
        (r#"const uri = "http://schemas.android.com/apk/res/123-invalid";"#, None),
        (r#"const uri = "http://maven.apache.org/xsd/maven-4.0.0.xsd";"#, None),
        (r#"const uri = "http://maven.apache.org/DOWNLOAD/1.0";"#, None),
        (r#"const uri = "http://www.springframework.org/schema/beans/spring-beans.xsd";"#, None),
        (r#"const uri = "http://www.springframework.org/schema/beans?version=1";"#, None),
        (r#"const uri = "http://purl.org/dc/terms/title/document";"#, None),
        (r#"const uri = "http://purl.org/dc/terms/title?format=xml";"#, None),
        (r#"const uri = "http://purl.org/dc/terms/title.";"#, None),
        (r#"const uri = "http://schemas.android.com/apk/res/android.";"#, None),
    ];

    let fix = vec![
        (r#"const url = "http://sindresorhus.com";"#, r#"const url = "https://sindresorhus.com";"#),
        (
            r#"const url = "http://sindresorhus.com:8080/path";"#,
            r#"const url = "https://sindresorhus.com:8080/path";"#,
        ),
    ];

    Tester::new(PreferHttps::NAME, PreferHttps::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName, JSXElementName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_html_link_for_pages_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `<a>` elements to navigate between Next.js pages.")
        .with_help("Use `<Link />` from `next/link` instead for internal navigation. See https://nextjs.org/docs/messages/no-html-link-for-pages")
        .with_label(span.label("Replace with `<Link>` from `next/link`"))
}

#[derive(Debug, Default, Clone)]
pub struct NoHtmlLinkForPages;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents the usage of `<a>` elements to navigate between Next.js pages.
    ///
    /// ### Why is this bad?
    ///
    /// Using `<a>` elements for internal navigation in Next.js applications can cause:
    /// - Full page reloads instead of client-side navigation
    /// - Loss of application state
    /// - Slower navigation performance
    /// - Broken prefetching capabilities
    ///
    /// Next.js provides the `<Link />` component from `next/link` for client-side navigation
    /// between pages, which provides better performance and user experience.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function HomePage() {
    ///   return (
    ///     <div>
    ///       <a href="/about">About Us</a>
    ///       <a href="/contact">Contact</a>
    ///     </div>
    ///   );
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import Link from 'next/link';
    ///
    /// function HomePage() {
    ///   return (
    ///     <div>
    ///       <Link href="/about">About Us</Link>
    ///       <Link href="/contact">Contact</Link>
    ///     </div>
    ///   );
    /// }
    /// ```
    ///
    /// External links are allowed:
    /// ```jsx
    /// function HomePage() {
    ///   return (
    ///     <div>
    ///       <a href="https://example.com">External Link</a>
    ///       <a href="mailto:contact@example.com">Email</a>
    ///       <a href="tel:+1234567890">Phone</a>
    ///     </div>
    ///   );
    /// }
    /// ```
    NoHtmlLinkForPages,
    nextjs,
    correctness
);

impl Rule for NoHtmlLinkForPages {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_element) = node.kind() else {
            return;
        };

        let JSXElementName::Identifier(jsx_element_name) = &jsx_opening_element.name else {
            return;
        };

        // Only check <a> elements
        if jsx_element_name.name != "a" {
            return;
        }

        // Find the href attribute
        let href_attr = jsx_opening_element.attributes.iter().find_map(|attr| {
            if let JSXAttributeItem::Attribute(attr) = attr
                && let JSXAttributeName::Identifier(name) = &attr.name
                && name.name == "href"
            {
                return Some(attr);
            }
            None
        });

        let Some(href_attr) = href_attr else {
            return;
        };

        // Check if href value indicates an internal link
        let is_internal_link = href_attr.value.as_ref().is_some_and(|value| {
            match value {
                // String literal href
                oxc_ast::ast::JSXAttributeValue::StringLiteral(str_lit) => {
                    let href_value = str_lit.value.as_str();
                    is_internal_page_link(href_value)
                }
                // Expression href - we'll be conservative and flag it as potentially internal
                oxc_ast::ast::JSXAttributeValue::ExpressionContainer(_) => true,
                _ => false,
            }
        });

        if is_internal_link {
            ctx.diagnostic(no_html_link_for_pages_diagnostic(jsx_opening_element.span));
        }
    }
}

/// Determines if an href value represents an internal page link
fn is_internal_page_link(href: &str) -> bool {
    // Skip external links
    if href.starts_with("http://") || href.starts_with("https://") {
        return false;
    }

    // Skip protocol-relative URLs
    if href.starts_with("//") {
        return false;
    }

    // Skip other protocols
    if href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("ftp:")
        || href.starts_with("file:")
    {
        return false;
    }

    // Skip hash links (same page)
    if href.starts_with('#') {
        return false;
    }

    // Skip empty href
    if href.is_empty() {
        return false;
    }

    // Internal links typically start with / or are relative paths
    href.starts_with('/')
        || (!href.split('/').next().unwrap_or("").contains(':') && !href.starts_with("//"))
}

#[test]
fn test_is_internal_page_link() {
    // Internal links
    assert!(is_internal_page_link("/about"));
    assert!(is_internal_page_link("/contact/us"));
    assert!(is_internal_page_link("about"));
    assert!(is_internal_page_link("../contact"));
    assert!(is_internal_page_link("./about"));

    // External links
    assert!(!is_internal_page_link("https://example.com"));
    assert!(!is_internal_page_link("http://example.com"));
    assert!(!is_internal_page_link("mailto:test@example.com"));
    assert!(!is_internal_page_link("tel:+1234567890"));
    assert!(!is_internal_page_link("ftp://example.com"));
    assert!(!is_internal_page_link("file://path/to/file"));

    // Hash links (same page)
    assert!(!is_internal_page_link("#section"));
    assert!(!is_internal_page_link("#"));

    // Empty href
    assert!(!is_internal_page_link(""));

    // Protocol-relative URLs
    assert!(!is_internal_page_link("//example.com"));
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // External links are allowed
        r"<a href='https://example.com'>External Link</a>",
        r"<a href='http://example.com'>External Link</a>",
        r"<a href='mailto:contact@example.com'>Email</a>",
        r"<a href='tel:+1234567890'>Phone</a>",
        r"<a href='ftp://example.com'>FTP</a>",
        r"<a href='file://path/to/file'>File</a>",
        // Hash links are allowed
        r"<a href='#section'>Jump to section</a>",
        r"<a href='#'>Empty hash</a>",
        // Protocol-relative URLs are allowed
        r"<a href='//example.com'>Protocol-relative</a>",
        // Links without href
        r"<a>No href</a>",
        // Other elements
        r"<div href='/about'>Not an anchor</div>",
        // Next.js Link component (correct usage)
        r"<Link href='/about'>About</Link>",
    ];

    let fail = vec![
        // Internal page links
        r"<a href='/about'>About</a>",
        r"<a href='/contact/us'>Contact Us</a>",
        r"<a href='about'>About</a>",
        r"<a href='../contact'>Contact</a>",
        r"<a href='./about'>About</a>",
        // Dynamic hrefs (expressions)
        r"<a href={dynamicLink}>Dynamic</a>",
        r"<a href={`/user/${userId}`}>User Profile</a>",
    ];

    Tester::new(NoHtmlLinkForPages::NAME, NoHtmlLinkForPages::PLUGIN, pass, fail)
        .test_and_snapshot();
}

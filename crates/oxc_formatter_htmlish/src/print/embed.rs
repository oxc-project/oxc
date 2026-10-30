//! Content written in another language, formatted by the session's dispatcher:
//! `<script>` and `<style>` bodies, `style` and event handler attribute values.
//! A part the dispatcher does not format prints as written.

use std::any::Any;

use oxc_allocator::{ArenaVec, StringBuilder};
use oxc_formatter_core::{
    FormatElement, FormatOptions, builders::FormatOnce, dispatch_fragment_ir, map_text_in_ir, write,
};
use oxc_htmlish_parser::ast::NodeId;

use super::{
    HtmlishFormatter,
    attribute::{quoted_piece, write_expanded},
    text::dedent_string,
};
use crate::tree::Tree;

/// Parent→child context for CSS dispatched from an HTML `style` attribute's value (css-in-html).
///
/// The host translates it into CSS's `CssEmbeddedIn`.
pub struct CssInHtmlStyleAttribute;

/// Parent→child context for JS dispatched from an HTML event handler attribute's value (js-in-html, `onclick="..."`).
///
/// The host translates it into JS's `JsEmbeddedIn`.
pub struct JsInHtmlEventHandler;

/// The event handler attributes (Prettier's `@prettier/html-event-attributes`).
const EVENT_HANDLERS: [&str; 90] = [
    "onabort",
    "onafterprint",
    "onauxclick",
    "onbeforeinput",
    "onbeforematch",
    "onbeforeprint",
    "onbeforetoggle",
    "onbeforeunload",
    "onblur",
    "oncancel",
    "oncanplay",
    "oncanplaythrough",
    "onchange",
    "onclick",
    "onclose",
    "oncommand",
    "oncontextlost",
    "oncontextmenu",
    "oncontextrestored",
    "oncopy",
    "oncuechange",
    "oncut",
    "ondblclick",
    "ondrag",
    "ondragend",
    "ondragenter",
    "ondragleave",
    "ondragover",
    "ondragstart",
    "ondrop",
    "ondurationchange",
    "onemptied",
    "onended",
    "onerror",
    "onfocus",
    "onformdata",
    "onhashchange",
    "oninput",
    "oninvalid",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onlanguagechange",
    "onload",
    "onloadeddata",
    "onloadedmetadata",
    "onloadstart",
    "onmessage",
    "onmessageerror",
    "onmousedown",
    "onmouseenter",
    "onmouseleave",
    "onmousemove",
    "onmouseout",
    "onmouseover",
    "onmouseup",
    "onoffline",
    "ononline",
    "onpagehide",
    "onpagereveal",
    "onpageshow",
    "onpageswap",
    "onpaste",
    "onpause",
    "onplay",
    "onplaying",
    "onpopstate",
    "onprogress",
    "onratechange",
    "onrejectionhandled",
    "onreset",
    "onresize",
    "onscroll",
    "onscrollend",
    "onsecuritypolicyviolation",
    "onseeked",
    "onseeking",
    "onselect",
    "onslotchange",
    "onstalled",
    "onstorage",
    "onsubmit",
    "onsuspend",
    "ontimeupdate",
    "ontoggle",
    "onunhandledrejection",
    "onunload",
    "onvolumechange",
    "onwaiting",
    "onwheel",
];

/// An event handler attribute (`onclick`), whose value is JS.
/// The name matches as written, as Prettier does (`onClick` stays as written).
pub fn is_event_handler(name: &str) -> bool {
    name.starts_with("on") && EVENT_HANDLERS.contains(&name)
}

/// Writes an attribute's (unescaped) `value` formatted in `language`, quoted, a `"` in it as `&quot;`;
/// `false` when the dispatcher leaves it as written.
pub fn write_attribute_value<'a>(
    language: &str,
    parent_context: &dyn Any,
    value: &'a str,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let Some(ir) = dispatch_fragment_ir(f, language, value, Some(parent_context)) else {
        return false;
    };
    if ir.is_empty() {
        write!(f, "\"\"");
        return true;
    }
    let indent_width = f.options().indent_width();
    let ir = map_text_in_ir(ir, f.allocator(), &mut |text, out| {
        if !text.contains('"') {
            return false;
        }
        out.push(FormatElement::text(quoted_piece(text, f), indent_width));
        true
    });
    write_expanded(&FormatOnce::new(|f: &mut HtmlishFormatter<'_, 'a>| f.write_elements(ir)), f);
    true
}

/// Writes the body of `<script>` / `<style>` element `id` formatted in its language;
/// `false` when it has none or the dispatcher leaves it as written.
pub fn write_embedded_content<'a>(
    t: &'a Tree<'a>,
    id: NodeId,
    value: &'a str,
    f: &mut HtmlishFormatter<'_, 'a>,
) -> bool {
    let Some(language) = content_language(t, id) else { return false };
    // A Markdown body starts at the line after the tag, dedented (Prettier's `dedentString`)
    let value = if language == "markdown" {
        let value = value
            .find('\n')
            .filter(|&lf| value[..lf].trim_ascii().is_empty())
            .map_or(value, |lf| &value[lf + 1..]);
        dedent_string(value, f)
    } else {
        value
    };
    let Some(ir) = dispatch_fragment_ir(f, language, value, None) else { return false };
    let ir = if t.tag(id) == "script" { escape_script_end(ir, f) } else { ir };
    f.write_elements(ir);
    true
}

/// The first `</script` ends a `<script>` body, so the source body holds none:
/// one in the formatted body is the formatting's own (a template literal's re-cooked part, e.g. html-in-js),
/// printed as `<\/script`, where `\/` means `/` (in a JS string or template, and in JSON).
///
/// A nested HTML part prints its end tags as one text (`tag.rs`), so none is split across texts.
fn escape_script_end<'a>(
    ir: ArenaVec<'a, FormatElement<'a>>,
    f: &HtmlishFormatter<'_, 'a>,
) -> ArenaVec<'a, FormatElement<'a>> {
    let allocator = f.allocator();
    let indent_width = f.options().indent_width();
    map_text_in_ir(ir, allocator, &mut |text, out| {
        let is_script_end = |i: usize| {
            let bytes = &text.as_bytes()[i + 2..];
            bytes.get(..6).is_some_and(|name| name.eq_ignore_ascii_case(b"script"))
                && !bytes.get(6).is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
        };
        let mut ends = text.match_indices("</").filter(|&(i, _)| is_script_end(i)).peekable();
        if ends.peek().is_none() {
            return false;
        }
        let mut escaped = StringBuilder::new_in(allocator);
        let mut run_start = 0;
        for (i, _) in ends {
            escaped.push_str(&text[run_start..i]);
            escaped.push_str("<\\/");
            run_start = i + 2;
        }
        escaped.push_str(&text[run_start..]);
        out.push(FormatElement::text(escaped.into_str(), indent_width));
        true
    })
}

/// The language of the body of `<script>` / `<style>` element `id`, a name the dispatcher resolves
/// (Prettier's `inferScriptParser` / `inferStyleParser`); `None` for none (`<script src>`, a template type).
fn content_language<'a>(t: &'a Tree<'a>, id: NodeId) -> Option<&'a str> {
    let el = t.element(id)?;
    let attribute = |name: &str| {
        el.attributes
            .iter()
            .find(|attribute| t.attribute_is(attribute.name.slice(t.source()), name))
    };
    let value =
        |name: &str| attribute(name).map(|a| a.value.map_or("", |v| v.span.slice(t.source())));
    match t.tag(id) {
        "style" => Some(value("lang").unwrap_or("css")),
        "script" if attribute("src").is_none() => match (value("lang"), value("type")) {
            (None, None) => Some("js"),
            (Some(lang), _) => Some(lang),
            (None, Some(kind)) => script_type_language(kind),
        },
        _ => None,
    }
}

/// A `<script type>` (Prettier's `inferParserByTypeAttribute`).
fn script_type_language(kind: &str) -> Option<&str> {
    Some(match kind {
        "module" | "text/javascript" | "application/javascript" => "js",
        "text/babel" | "text/jsx" => "jsx",
        "application/x-typescript" => "ts",
        "text/markdown" => "markdown",
        "text/html" => "html",
        "text/x-handlebars-template" => "handlebars",
        _ if kind.ends_with("json")
            || kind.ends_with("importmap")
            || kind == "speculationrules" =>
        {
            "json"
        }
        _ => return None,
    })
}

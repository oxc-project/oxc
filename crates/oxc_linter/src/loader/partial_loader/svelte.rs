use memchr::{memchr2, memmem::Finder};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_syntax::{
    identifier::{is_identifier_part, is_identifier_start, is_white_space},
    line_terminator::is_line_terminator,
};

use crate::loader::JavaScriptSource;

use super::{AttributeValue, COMMENT_END, COMMENT_START, SCRIPT_END, SCRIPT_START, find_attribute};

pub struct SveltePartialLoader<'a> {
    source_text: &'a str,
}

impl<'a> SveltePartialLoader<'a> {
    pub fn new(source_text: &'a str) -> Self {
        Self { source_text }
    }

    pub fn parse(self) -> Vec<JavaScriptSource<'a>> {
        self.parse_scripts()
    }

    /// Each *.svelte file can contain at most
    ///  * one `<script>` block
    ///  * one `<script context="module">` or `<script module>` block
    ///    <https://github.com/sveltejs/svelte.dev/blob/ba7ad256f786aa5bc67eac3a58608f3f50b59e91/apps/svelte.dev/content/tutorial/02-advanced-svelte/08-script-module/02-module-exports/index.md>
    fn parse_scripts(&self) -> Vec<JavaScriptSource<'a>> {
        let mut pointer = 0;
        let mut context = MarkupContext::default();
        let Some(result1) = self.parse_script(&mut pointer, &mut context) else {
            return vec![];
        };
        let Some(result2) = self.parse_script(&mut pointer, &mut context) else {
            return vec![result1];
        };
        vec![result1, result2]
    }

    fn parse_script(
        &self,
        pointer: &mut usize,
        context: &mut MarkupContext<'a>,
    ) -> Option<JavaScriptSource<'a>> {
        let script_end_finder = Finder::new(SCRIPT_END);
        *pointer = self.find_script_start(*pointer, context)?;

        let end = find_markup_tag_end(self.source_text, *pointer, context.is_root())?.end;
        let offset = end - *pointer - 1;

        let content = &self.source_text[*pointer..*pointer + offset];
        let lang = Self::extract_lang_attribute(content);
        let mut source_type =
            SourceType::from_extension(lang).unwrap_or_else(|_| SourceType::mjs());

        // Keep existing behavior for plain `<script>` blocks while detecting `lang="ts"`.
        // In Svelte, module semantics are controlled by `module`/`context="module"`.
        if source_type.is_unambiguous() || Self::is_module_script(content) {
            source_type = source_type.with_module(true);
        }

        *pointer += offset + 1;
        let js_start = *pointer;

        // find "</script>"
        let offset = script_end_finder.find(&self.source_text.as_bytes()[*pointer..])?;
        let js_end = *pointer + offset;
        *pointer += offset + SCRIPT_END.len();

        let source_text = &self.source_text[js_start..js_end];

        // NOTE: loader checked that source_text.len() is less than u32::MAX
        #[expect(clippy::cast_possible_truncation)]
        Some(JavaScriptSource::partial(source_text, source_type, js_start as u32))
    }

    fn find_script_start(
        &self,
        mut pointer: usize,
        context: &mut MarkupContext<'a>,
    ) -> Option<usize> {
        if !self.source_text[pointer..].contains(SCRIPT_START) {
            return None;
        }
        let bytes = self.source_text.as_bytes();
        while let Some(offset) = memchr2(b'<', b'{', &bytes[pointer..]) {
            pointer += offset;
            let rest = &self.source_text[pointer..];
            if rest.starts_with('{') {
                if let Some(end) = skip_markup_expression(self.source_text, pointer) {
                    context.expression(&self.source_text[pointer + 1..end - 1]);
                    pointer = end;
                } else {
                    pointer += 1;
                }
            } else if rest.starts_with(COMMENT_START) {
                pointer += rest.find(COMMENT_END)? + COMMENT_END.len();
            } else {
                let tag = rest.strip_prefix("</").or_else(|| rest.strip_prefix('<'))?;
                if !tag.chars().next().is_some_and(is_identifier_start) {
                    pointer += 1;
                    continue;
                }
                let name_end =
                    tag.find(|c: char| is_svelte_whitespace(c) || matches!(c, '/' | '>'))?;
                let name = &tag[..name_end];
                let closing = rest.starts_with("</");
                let static_text = matches!(name, "script" | "style") && context.is_root();
                let Some(tag_end) = find_markup_tag_end(self.source_text, pointer + 1, static_text)
                else {
                    pointer += 1;
                    continue;
                };
                let end = tag_end.end;
                let self_closing = tag_end.self_closing;
                if !closing && name == "script" && !self_closing {
                    return Some(pointer + SCRIPT_START.len());
                }
                let component_style_attributes =
                    static_text.then_some(&self.source_text[pointer..end - 1]);
                pointer = end;
                if closing {
                    context.close(name);
                } else {
                    match name {
                        "style" if !self_closing => {
                            pointer =
                                skip_style(self.source_text, pointer, component_style_attributes)?;
                        }
                        "textarea" if !self_closing => {
                            pointer = skip_textarea(self.source_text, pointer)?;
                        }
                        _ => context.open(name, self_closing),
                    }
                }
            }
        }
        None
    }

    fn extract_lang_attribute(content: &str) -> &str {
        match find_attribute(content, "lang") {
            Some(AttributeValue::Value(lang)) if !lang.is_empty() => lang,
            _ => "mjs",
        }
    }

    fn is_module_script(content: &str) -> bool {
        find_attribute(content, "module").is_some()
            || matches!(find_attribute(content, "context"), Some(AttributeValue::Value("module")))
    }
}

#[derive(Default)]
struct MarkupContext<'a> {
    elements: Vec<&'a str>,
    blocks: Vec<usize>,
}

impl<'a> MarkupContext<'a> {
    fn is_root(&self) -> bool {
        self.elements.is_empty() && self.blocks.is_empty()
    }

    fn expression(&mut self, expression: &str) {
        match expression.trim_start_matches(is_svelte_whitespace).as_bytes().first() {
            Some(b'#') => self.blocks.push(self.elements.len()),
            Some(b':') => {
                if let Some(&depth) = self.blocks.last() {
                    self.elements.truncate(depth);
                }
            }
            Some(b'/') => {
                if let Some(depth) = self.blocks.pop() {
                    self.elements.truncate(depth);
                }
            }
            _ => {}
        }
    }

    fn close(&mut self, name: &str) {
        let depth = self.blocks.last().copied().unwrap_or(0);
        if let Some(index) = self.elements[depth..].iter().rposition(|&element| element == name) {
            self.elements.truncate(depth + index);
        }
    }

    fn open(&mut self, name: &'a str, self_closing: bool) {
        let depth = self.blocks.last().copied().unwrap_or(0);
        if self.elements.len() > depth
            && self.elements.last().is_some_and(|&parent| implicitly_closes(parent, name))
        {
            self.elements.pop();
        }
        if !self_closing
            && !matches!(
                name,
                "area"
                    | "base"
                    | "br"
                    | "col"
                    | "command"
                    | "embed"
                    | "hr"
                    | "img"
                    | "input"
                    | "keygen"
                    | "link"
                    | "meta"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            )
        {
            self.elements.push(name);
        }
    }
}

fn implicitly_closes(parent: &str, name: &str) -> bool {
    match parent {
        "li" => name == "li",
        "dt" | "dd" => matches!(name, "dt" | "dd"),
        "p" => matches!(
            name,
            "address"
                | "article"
                | "aside"
                | "blockquote"
                | "div"
                | "dl"
                | "fieldset"
                | "footer"
                | "form"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "header"
                | "hgroup"
                | "hr"
                | "main"
                | "menu"
                | "nav"
                | "ol"
                | "p"
                | "pre"
                | "section"
                | "table"
                | "ul"
        ),
        "rt" | "rp" => matches!(name, "rt" | "rp"),
        "optgroup" => name == "optgroup",
        "option" => matches!(name, "option" | "optgroup"),
        "thead" | "tbody" => matches!(name, "tbody" | "tfoot"),
        "tfoot" => name == "tbody",
        "tr" => matches!(name, "tr" | "tbody"),
        "td" | "th" => matches!(name, "td" | "th" | "tr"),
        _ => false,
    }
}

fn skip_textarea(source_text: &str, mut pointer: usize) -> Option<usize> {
    let bytes = source_text.as_bytes();
    while let Some(offset) = memchr2(b'<', b'{', &bytes[pointer..]) {
        pointer += offset;
        let rest = &source_text[pointer..];
        if rest.starts_with('{') {
            pointer = skip_markup_expression(source_text, pointer).unwrap_or(pointer + 1);
        } else if rest
            .as_bytes()
            .get(..10)
            .is_some_and(|name| name.eq_ignore_ascii_case(b"</textarea"))
            && (rest[10..].starts_with('>') || rest[10..].starts_with(is_svelte_whitespace))
        {
            return Some(pointer + 10 + rest[10..].find('>')? + 1);
        } else {
            pointer += 1;
        }
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttributePosition {
    Name,
    BeforeValue,
    UnquotedValue,
}

struct MarkupTagEnd {
    end: usize,
    self_closing: bool,
}

fn find_markup_tag_end(
    source_text: &str,
    mut pointer: usize,
    static_text: bool,
) -> Option<MarkupTagEnd> {
    let bytes = source_text.as_bytes();
    let mut quote = None;
    let mut self_closing = false;
    let mut position = AttributePosition::Name;
    while let Some(&byte) = bytes.get(pointer) {
        let character = source_text[pointer..].chars().next()?;
        if quote.is_none() {
            if !static_text && position == AttributePosition::Name {
                let rest = &source_text[pointer..];
                if let Some(comment) = rest.strip_prefix("/*") {
                    pointer += comment.find("*/")? + 4;
                    continue;
                }
                if let Some(comment) = rest.strip_prefix("//") {
                    pointer += comment.find('\n')? + 2;
                    continue;
                }
            }
            if is_svelte_whitespace(character) {
                if position == AttributePosition::UnquotedValue {
                    position = AttributePosition::Name;
                }
                pointer += character.len_utf8();
                continue;
            }
        }
        if byte == b'{'
            && (!static_text || quote.is_none())
            && let Some(end) = skip_attribute_expression(source_text, pointer)
        {
            if quote.is_none() && position == AttributePosition::BeforeValue {
                position = AttributePosition::UnquotedValue;
            }
            self_closing = false;
            pointer = end;
            continue;
        }
        let slash_closes = quote.is_none()
            && byte == b'/'
            && (position == AttributePosition::Name
                || !static_text
                    && position == AttributePosition::UnquotedValue
                    && bytes.get(pointer + 1) == Some(&b'>'));
        match byte {
            b'\'' | b'"' if quote == Some(byte) => {
                quote = None;
                position = AttributePosition::Name;
            }
            b'\'' | b'"' if quote.is_none() => {
                quote = Some(byte);
                position = AttributePosition::Name;
            }
            b'>' if quote.is_none() => {
                return Some(MarkupTagEnd { end: pointer + 1, self_closing });
            }
            b'=' if quote.is_none() && position == AttributePosition::Name => {
                position = AttributePosition::BeforeValue;
            }
            _ if quote.is_none() && position == AttributePosition::BeforeValue => {
                position = AttributePosition::UnquotedValue;
            }
            _ => {}
        }
        self_closing = slash_closes;
        pointer += character.len_utf8();
    }
    None
}

fn skip_style(
    source_text: &str,
    mut pointer: usize,
    component_attributes: Option<&str>,
) -> Option<usize> {
    let Some(attributes) = component_attributes else {
        return Some(pointer + source_text[pointer..].find("</style>")? + "</style>".len());
    };
    if find_attribute(attributes, "lang")
        .is_some_and(|lang| !matches!(lang, AttributeValue::Value("css")))
    {
        let finder = Finder::new("</style");
        loop {
            pointer += finder.find(&source_text.as_bytes()[pointer..])? + "</style".len();
            let rest = source_text[pointer..].trim_start_matches(is_svelte_whitespace);
            if rest.starts_with('>') {
                return Some(source_text.len() - rest.len() + 1);
            }
        }
    }
    let bytes = source_text.as_bytes();
    let mut braces = 0usize;
    let mut parentheses = 0usize;
    let mut brackets = 0usize;
    let mut quote = None;
    while let Some(&byte) = bytes.get(pointer) {
        if byte == b'\\' {
            pointer = pointer.saturating_add(2);
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' => quote = Some(byte),
                b'/' if bytes.get(pointer + 1) == Some(&b'*') => {
                    pointer += source_text[pointer + 2..].find("*/")? + 4;
                    continue;
                }
                b'(' if pointer >= 3
                    && bytes[pointer - 3..pointer].eq_ignore_ascii_case(b"url") =>
                {
                    pointer = skip_css_url(source_text, pointer + 1)?;
                    continue;
                }
                b'{' => braces += 1,
                b'}' => braces = braces.saturating_sub(1),
                b'(' => parentheses += 1,
                b')' => parentheses = parentheses.saturating_sub(1),
                b'[' => brackets += 1,
                b']' => brackets = brackets.saturating_sub(1),
                b'<' if source_text[pointer..].starts_with(COMMENT_START) => {
                    pointer += source_text[pointer + COMMENT_START.len()..].find(COMMENT_END)?
                        + COMMENT_START.len()
                        + COMMENT_END.len();
                    continue;
                }
                b'<' if braces == 0 && parentheses == 0 && brackets == 0 => {
                    if let Some(rest) = source_text[pointer..].strip_prefix("</style") {
                        let rest = rest.trim_start_matches(is_svelte_whitespace);
                        if rest.starts_with('>') {
                            return Some(source_text.len() - rest.len() + 1);
                        }
                    }
                }
                _ => {}
            }
        }
        pointer += 1;
    }
    None
}

fn skip_css_url(source_text: &str, mut pointer: usize) -> Option<usize> {
    let bytes = source_text.as_bytes();
    let mut quote = None;
    while let Some(&byte) = bytes.get(pointer) {
        if byte == b'\\' {
            pointer = pointer.saturating_add(2);
            continue;
        }
        match byte {
            b'\'' | b'"' if quote == Some(byte) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(byte),
            b')' if quote.is_none() => return Some(pointer + 1),
            _ => {}
        }
        pointer += 1;
    }
    None
}

fn is_svelte_whitespace(c: char) -> bool {
    is_white_space(c) || is_line_terminator(c)
}

fn skip_markup_expression(source_text: &str, pointer: usize) -> Option<usize> {
    let mut expression = source_text[pointer + 1..].trim_start_matches(is_svelte_whitespace);
    if let Some(closing_block) = expression.strip_prefix('/')
        && let Some(name_end) = closing_block.find(|c: char| !c.is_ascii_alphabetic())
        && name_end > 0
    {
        let rest = closing_block[name_end..].trim_start_matches(is_svelte_whitespace);
        if rest.starts_with('}') {
            return Some(source_text.len() - rest.len() + 1);
        }
    }
    let kind = if expression.starts_with("#each") {
        EmbeddedExpression::Each
    } else if expression.starts_with("#await") {
        EmbeddedExpression::Await
    } else if expression.starts_with("#snippet") {
        EmbeddedExpression::Snippet
    } else if expression.starts_with("@const")
        || expression.starts_with(":then")
        || expression.starts_with(":catch")
    {
        EmbeddedExpression::Binding
    } else if ["const", "let"].iter().any(|keyword| {
        expression
            .strip_prefix(keyword)
            .is_some_and(|rest| rest.chars().next().is_none_or(|ch| !is_identifier_part(ch)))
    }) {
        EmbeddedExpression::Declaration
    } else {
        EmbeddedExpression::Expression
    };
    let is_else = expression.starts_with(":else");
    if expression.starts_with(['@', '#', ':']) {
        expression = expression[1..]
            .trim_start_matches(|c: char| c.is_ascii_alphabetic())
            .trim_start_matches(is_svelte_whitespace);
        if is_else && let Some(condition) = expression.strip_prefix("if") {
            expression = condition.trim_start_matches(is_svelte_whitespace);
        }
        if expression.starts_with('}') {
            return Some(source_text.len() - expression.len() + 1);
        }
    }
    skip_embedded_expression(source_text, source_text.len() - expression.len(), kind)
}

fn skip_attribute_expression(source_text: &str, pointer: usize) -> Option<usize> {
    let expression = source_text[pointer + 1..].trim_start_matches(is_svelte_whitespace);
    let expression = expression
        .strip_prefix("...")
        .or_else(|| {
            expression.strip_prefix("@attach").filter(|rest| rest.starts_with(is_svelte_whitespace))
        })
        .unwrap_or(expression)
        .trim_start_matches(is_svelte_whitespace);
    skip_expression(source_text, source_text.len() - expression.len())
}

#[derive(Clone, Copy)]
enum EmbeddedExpression {
    Expression,
    Each,
    Await,
    Snippet,
    Binding,
    Declaration,
}

fn skip_expression(source_text: &str, pointer: usize) -> Option<usize> {
    skip_embedded_expression(source_text, pointer, EmbeddedExpression::Expression)
}

fn skip_embedded_expression(
    source_text: &str,
    pointer: usize,
    kind: EmbeddedExpression,
) -> Option<usize> {
    let source = &source_text[pointer..];
    let mut chunk_len = 128usize;
    loop {
        let mut end = chunk_len.min(source.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        let mut candidates = Vec::new();
        validate_embedded_expression(&source[..end], kind, &mut candidates);
        candidates.sort_unstable();
        candidates.dedup();
        for candidate in candidates {
            if source.as_bytes().get(candidate) == Some(&b'}')
                && validate_embedded_expression(&source[..candidate], kind, &mut Vec::new())
            {
                return Some(pointer + candidate + 1);
            }
        }
        if end == source.len() {
            return memchr::memchr_iter(b'}', source.as_bytes()).find_map(|candidate| {
                validate_embedded_expression(&source[..candidate], kind, &mut Vec::new())
                    .then_some(pointer + candidate + 1)
            });
        }
        chunk_len = chunk_len.saturating_mul(2);
    }
}

fn validate_embedded_expression(
    source: &str,
    kind: EmbeddedExpression,
    candidates: &mut Vec<usize>,
) -> bool {
    if matches!(kind, EmbeddedExpression::Declaration) {
        if validate_expression(source, "async () => {", ";}", 0, candidates) {
            return true;
        }
        validate_expression(source, "", "", 0, candidates);
        return false;
    }
    if matches!(kind, EmbeddedExpression::Snippet) {
        return validate_expression(source, "function ", " {}", 0, candidates);
    }
    if matches!(kind, EmbeddedExpression::Binding) {
        return validate_expression(source, "(", ") => 0", 0, candidates);
    }
    if validate_expression(source, "", "", 0, candidates) {
        return true;
    }
    if has_expression_boundary(source, kind, candidates) {
        return false;
    }
    let Some(first_error) = candidates.iter().copied().min() else {
        return false;
    };
    if source.as_bytes().get(first_error).is_some_and(|byte| matches!(byte, b'\'' | b'"' | b'`')) {
        return false;
    }
    let separators: &[&str] = match kind {
        EmbeddedExpression::Each => &["as"],
        EmbeddedExpression::Await => &["then", "catch"],
        _ => return false,
    };
    for separator in separators {
        let mut offsets: Vec<_> =
            source.match_indices(separator).map(|(offset, _)| (offset, false)).collect();
        offsets.sort_unstable_by_key(|&(offset, _)| {
            (offset > first_error, offset.abs_diff(first_error))
        });
        let mut opaque_ranges = Vec::new();
        let mut index = 0;
        while let Some(offset) = next_header_candidate(&mut offsets, &mut index, &opaque_ranges, 0)
        {
            let after = &source[offset + separator.len()..];
            if source[..offset].chars().next_back().is_some_and(is_identifier_part)
                || !(after.starts_with(is_svelte_whitespace)
                    || matches!(kind, EmbeddedExpression::Await)
                        && (after.is_empty() || after.starts_with('}')))
            {
                continue;
            }
            let mut errors = Vec::new();
            if !validate_expression(&source[..offset], "", "", 0, &mut errors) {
                opaque_ranges.extend(
                    errors
                        .into_iter()
                        .filter_map(|position| opaque_expression_range(source, position)),
                );
                if opaque_ranges.iter().any(|range| range.end == source.len()) {
                    return false;
                }
                continue;
            }
            let binding_start = offset + separator.len();
            let binding = &source[binding_start..];
            let binding_error_start = candidates.len();
            if validate_expression(binding, "(", ") => 0", binding_start, candidates) {
                return true;
            }
            opaque_ranges.extend(
                candidates[binding_error_start..]
                    .iter()
                    .filter_map(|&position| opaque_expression_range(source, position)),
            );
            if opaque_ranges.iter().any(|range| range.end == source.len()) {
                return false;
            }
            if has_expression_boundary(source, kind, candidates) {
                return false;
            }
            if matches!(kind, EmbeddedExpression::Each) {
                let mut keys: Vec<_> =
                    binding.match_indices('(').map(|(offset, _)| (offset, false)).collect();
                keys.sort_unstable_by_key(|&(key_start, _)| {
                    !candidates.contains(&(binding_start + key_start))
                });
                let mut key_index = 0;
                while let Some(key_start) =
                    next_header_candidate(&mut keys, &mut key_index, &opaque_ranges, binding_start)
                {
                    let mut errors = Vec::new();
                    if !validate_expression(
                        &binding[..key_start],
                        "(",
                        ") => 0",
                        binding_start,
                        &mut errors,
                    ) {
                        opaque_ranges.extend(
                            errors
                                .into_iter()
                                .filter_map(|position| opaque_expression_range(source, position)),
                        );
                        if opaque_ranges.iter().any(|range| range.end == source.len()) {
                            return false;
                        }
                        continue;
                    }
                    let key = &binding[key_start..];
                    if validate_expression(key, "", "", binding_start + key_start, candidates) {
                        return true;
                    }
                    if has_expression_boundary(source, kind, candidates) {
                        return false;
                    }
                }
            }
        }
    }
    false
}

fn next_header_candidate(
    offsets: &mut Vec<(usize, bool)>,
    index: &mut usize,
    opaque_ranges: &[std::ops::Range<usize>],
    base: usize,
) -> Option<usize> {
    while let Some(&(offset, deferred)) = offsets.get(*index) {
        *index += 1;
        if !deferred && opaque_ranges.iter().any(|range| range.contains(&(base + offset))) {
            offsets.push((offset, true));
        } else {
            return Some(offset);
        }
    }
    None
}

fn opaque_expression_range(source: &str, pointer: usize) -> Option<std::ops::Range<usize>> {
    let rest = source.get(pointer..)?;
    if let Some(comment) = rest.strip_prefix("/*") {
        return Some(pointer..comment.find("*/").map_or(source.len(), |end| pointer + 2 + end + 2));
    }
    let quote = *source.as_bytes().get(pointer)?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let mut end = pointer + 1;
    while let Some(&byte) = source.as_bytes().get(end) {
        if byte == b'\\' {
            end += 2;
        } else if byte == quote {
            return Some(pointer..end + 1);
        } else {
            end += 1;
        }
    }
    Some(pointer..source.len())
}

fn has_expression_boundary(source: &str, kind: EmbeddedExpression, candidates: &[usize]) -> bool {
    candidates.iter().any(|&candidate| {
        source.as_bytes().get(candidate) == Some(&b'}')
            && validate_embedded_expression(&source[..candidate], kind, &mut Vec::new())
    })
}

fn validate_expression(
    source: &str,
    prefix: &str,
    suffix: &str,
    offset: usize,
    candidates: &mut Vec<usize>,
) -> bool {
    let allocator = Allocator::default();
    let wrapped;
    let input = if prefix.is_empty() && suffix.is_empty() {
        source
    } else {
        wrapped = format!("{prefix}{source}{suffix}");
        &wrapped
    };
    let diagnostics = if prefix.is_empty() && suffix.is_empty() {
        match Parser::new(&allocator, input, SourceType::ts()).parse_expression() {
            Ok(_) => return true,
            Err(mut diagnostics) => {
                if validate_expression(source, "async () => (", ")", offset, candidates) {
                    return true;
                }
                diagnostics
                    .extend(Parser::new(&allocator, input, SourceType::ts()).parse().diagnostics);
                diagnostics
            }
        }
    } else {
        let parsed = Parser::new(&allocator, input, SourceType::ts()).parse();
        // Svelte permits await in binding defaults, a recoverable error in parameter syntax.
        if !parsed.fatal_error && parsed.program.body.len() == 1 {
            return true;
        }
        parsed.diagnostics
    };
    for diagnostic in &diagnostics {
        for label in &diagnostic.labels {
            if let Some(position) = (label.offset() as usize).checked_sub(prefix.len())
                && position < source.len()
            {
                candidates.push(offset + position);
            }
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::{JavaScriptSource, SveltePartialLoader};

    fn parse_svelte(source_text: &str) -> JavaScriptSource<'_> {
        let sources = SveltePartialLoader::new(source_text).parse();
        *sources.first().unwrap()
    }

    #[test]
    fn test_parse_svelte() {
        let source_text = r#"
        <script>
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_script_inside_code_comment() {
        let source_text = r"
        <!-- <script>a</script> -->
        <!-- <script> -->
        <script>b</script>
        ";

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text, "b");
        assert_eq!(result.start, 79);
    }

    #[test]
    fn test_parse_svelte_ts_with_generic() {
        let source_text = r#"
        <script lang="ts" generics="T extends Record<string, unknown>">
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_with_module_script() {
        let source_text = r#"
        <script module>
          export async function load() { /* some loading logic */ }
        </script>
        <script>
          console.log("hi");
        </script>
        <h1>Hello World</h1>
        "#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources[0].source_text.trim(),
            "export async function load() { /* some loading logic */ }"
        );
        assert_eq!(sources[1].source_text.trim(), r#"console.log("hi");"#);
        assert!(sources[0].source_type.is_module());
        assert!(sources[1].source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_with_context_module_script() {
        let source_text = r#"
        <script context="module">
          debugger;
        </script>
        <script>
          console.log("hi");
        </script>
        "#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "debugger;");
        assert_eq!(sources[1].source_text.trim(), r#"console.log("hi");"#);
        assert!(sources[0].source_type.is_module());
        assert!(sources[1].source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_module_script_lang_ts() {
        let source_text = r"
        <script module lang='ts'>
          debugger;
        </script>
        ";

        let result = parse_svelte(source_text);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_context_module_script_lang_ts() {
        let source_text = r#"
        <script context="module" lang="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_does_not_treat_data_language_as_lang() {
        let source_text = r#"
        <script data-language="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert_eq!(result.source_text.trim(), "debugger;");
    }

    #[test]
    fn test_parse_svelte_ignores_script_like_tags() {
        let source_text = r#"
        <script-setup>
          debugger;
        </script-setup>
        <script>
          console.log("hi");
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), r#"console.log("hi");"#);
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_tag_allows_newline_after_script_name() {
        let source_text = r#"
        <script
          lang="ts">
          debugger;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_tag_allows_tab_after_script_name() {
        let source_text = "<script\tlang=\"ts\">\n  debugger;\n</script>\n";

        let result = parse_svelte(source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_typescript());
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_tag_with_expression_lang_and_gt_in_other_attribute() {
        let source_text = r#"
        <script
            lang={"ts"}
            accesskey=">"
            >
            let scoops = 1;
            let flavours = [];
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert!(result.source_text.contains("let scoops = 1;"));
    }

    #[test]
    fn test_parse_svelte_script_tag_with_spaced_expression_lang_is_not_dropped() {
        let source_text = r#"
        <script
            lang={ "ts" }
            >
            let scoops = 1;
        </script>
        "#;

        let result = parse_svelte(source_text);
        assert!(!result.source_type.is_typescript());
        assert!(result.source_type.is_module());
        assert!(result.source_text.contains("let scoops = 1;"));
    }

    #[test]
    fn test_parse_svelte_with_many_script_like_tags() {
        let mut source_text = "<script-setup>noop</script-setup>\n".repeat(2_000);
        source_text.push_str("<script>debugger;</script>");

        let result = parse_svelte(&source_text);
        assert_eq!(result.source_text.trim(), "debugger;");
        assert!(result.source_type.is_module());
    }

    #[test]
    fn test_parse_svelte_script_with_callback_attribute() {
        let source_text = r#"<script>
let browser = true;
</script>
{#if browser}
  <script src="/" onload={() => {}}></script>
{/if}"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].source_text.trim(), "let browser = true;");
        assert_eq!(sources[1].source_text, "");
    }

    #[test]
    fn test_parse_svelte_script_with_callback_attribute_no_component_script() {
        let source_text = r#"{#if browser}
  <script src="/" onload={() => {}}></script>
{/if}"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "");
    }
    #[test]
    fn test_script_in_html_expression() {
        let source_text = r#"<script lang="ts">
let {json}: {json: string} = $props();
</script>
<svelte:head>
    {@html `<script type="application/ld+json">${json}</script>`}
</svelte:head>"#;

        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text.trim(), "let {json}: {json: string} = $props();");
        assert!(sources[0].source_type.is_typescript());
    }

    #[test]
    fn test_html_expression_before_component_scripts() {
        let expressions = [
            r"{@html `<script>${json}</script>`}",
            r"{@html '<script>ignored</script>'}",
            r#"{@html "<script>ignored</script>"}"#,
            r#"{@html `<script>${`nested ${"}"}`}</script>`}"#,
            r#"{@html `<script>${({value: "}"}).value}</script>`}"#,
            r#"{@html "escaped \" <script>ignored</script>"}"#,
            r"{@html `escaped \` <script>ignored</script>`}",
            r"{@html /* } <script>ignored</script> */ html}",
            "{@html // } <script>ignored</script>\n html}",
            r"{@html /[{}]/.test(value) ? '<script>ignored</script>' : ''}",
            r"{@html count++ / 2}",
            r"{@html object.in / 2}",
            r"{@html 值 / 2}",
            r"{'{@html '}",
            r#"<style>.x::before { content: "{@html "; }</style>"#,
            r"<style>.x { color: red }</style >",
            "<textarea>hello</textarea\n>",
            r"<textarea />",
            r"<style />",
            r"<textarea>text </textarea-like> {@html </textarea>",
            r#"<div title="{@html ">content</div>"#,
            r#"<div title="{">content</div>"#,
            r"<textarea>{@html </textarea>",
            "{@html value // comment\u{2028}}",
            "{@html value // comment\u{2029}}",
            r"{@html value / count}",
            r"{@html `${/<script>}/.source}` }",
            r"<!-- {@html ` --> {@html '<script>ignored</script>'}",
        ];

        for expression in expressions {
            let source_text = format!(
                "<p>日本語</p>{expression}<script>debugger;</script><script module>export {{}};</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 2, "{expression}");
            assert_eq!(sources[0].source_text, "debugger;", "{expression}");
            assert_eq!(sources[1].source_text, "export {};", "{expression}");
            assert_eq!(sources[0].start as usize, source_text.find("debugger;").unwrap());
        }
    }

    #[test]
    fn test_html_expression_without_component_scripts() {
        let source_text = r"{@html `<script>${json}</script>`}";
        assert!(SveltePartialLoader::new(source_text).parse().is_empty());
    }
    #[test]
    fn test_script_after_attribute_with_braces_in_string() {
        for attributes in [
            r#"title={"}"}"#,
            r#"title={"{"}"#,
            r#"title={`${"}"}`}"#,
            r#"title="before {"}"} after""#,
            r#"{...{title: "}"}}"#,
            r#"{...(count > 0 ? {title: "}"} : {})}"#,
        ] {
            let source_text =
                format!(r#"<div {attributes}></div><script>const text = `">`; debugger;</script>"#);
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{attributes}");
            assert_eq!(sources[0].source_text, r#"const text = `">`; debugger;"#);
        }
    }

    #[test]
    fn test_script_after_closing_block() {
        for closing in ["{/if}", "{\u{feff}/if}", "{/if\u{feff}}"] {
            let source_text = format!(
                "{{#if true}}<p>hello</p>{closing}<script>const re = /}}/; debugger;</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{closing}");
            assert_eq!(sources[0].source_text, "const re = /}/; debugger;");
        }
    }

    #[test]
    fn test_script_with_literal_braces_in_component_attributes() {
        for source_text in [
            r#"<script data-x="{">const s="}"; debugger;</script>"#,
            r#"<script data-x="{'">const x = '}'; debugger;</script>"#,
            r#"<style data-x="{">.x { color: red; }</style><script>debugger;</script>"#,
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert!(sources[0].source_text.contains("debugger;"));
        }
    }

    #[test]
    fn test_regex_in_attribute_is_not_a_closing_block() {
        for attributes in [
            r#"title={/if}>/.test(value)} data-x="<script>ignored</script>""#,
            r#"title={/if}"/.test('<script>ignored</script>')}"#,
        ] {
            let source_text = format!("<div {attributes}></div><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{attributes}");
            assert_eq!(sources[0].source_text, "debugger;");
        }
    }
    #[test]
    fn test_script_after_else_if_header() {
        let source_text = "{#if ok}<p>one</p>{:else if value === '<textarea>'}<p>two</p>{/if}<script>debugger;</script>";
        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "debugger;");
    }

    #[test]
    fn test_script_after_other_block_headers() {
        for source_text in [
            "{#await value === '<textarea>' ? promise : promise then result}<p>hi</p>{/await}<script lang='ts'>debugger;</script>",
            "{#each items as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#await promise then {x = '<textarea>'}}<p>hi</p>{/await}<script lang='ts'>debugger;</script>",
            "{#each items as {x = '<textarea>'}, i ('<textarea>')}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#snippet content(x: string = '<textarea>')}<p>hi</p>{/snippet}<script lang='ts'>debugger;</script>",
            "{#each (items as string[]) as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
            "{#each `a${value as string}` as {x = '<textarea>'}}<p>hi</p>{/each}<script lang='ts'>debugger;</script>",
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;");
        }
    }
    #[test]
    fn test_script_after_awaited_markup_expressions() {
        for markup in [
            "{await Promise.resolve('<textarea>')}",
            "{@html await Promise.resolve('<textarea>')}",
            "{#if await Promise.resolve('<textarea>')}<p>hi</p>{/if}",
            "{#each await Promise.resolve(['<textarea>']) as value}<p>{value}</p>{/each}",
            "<div title={await Promise.resolve('><textarea>')}></div>",
            "<div title=\"before {await Promise.resolve('><textarea>')} after\"></div>",
        ] {
            let source_text = format!("{markup}<script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_script_after_attachment_attributes() {
        for attributes in [
            "{@attach fn(`><textarea>`)}",
            "{@attach fn({value: '><textarea>'})}",
            "{@attach\u{feff}fn(`><textarea>`)}",
            "{@attach /* } > <textarea> */ fn(`><textarea>`)}",
            "title=\"value\" {@attach fn(`><textarea>`)} data-value={42}",
        ] {
            let source_text = format!("<div {attributes}></div><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_static_script_attribute_slash_is_not_self_closing() {
        for attributes in ["data-x=foo/", "data-x=foo/ ", "data-x=/", "data-x=//literal/"] {
            let source_text = format!("<script {attributes}>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_attribute_comment_ending_is_not_self_closing() {
        for attributes in ["/* comment */", "title=\"value\" /* ' */", "// comment /\n"] {
            let source_text = format!(
                "<svelte:head><script {attributes}>nested();</script></svelte:head><script>component();</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 2, "{source_text}");
            assert_eq!(sources[0].source_text, "nested();", "{source_text}");
            assert_eq!(sources[1].source_text, "component();", "{source_text}");
        }
    }

    #[test]
    fn test_script_after_attribute_comments() {
        for attributes in [
            "/* '<textarea> */",
            "// '<textarea>\n",
            "// '<textarea>\r\n",
            "title=\"value\"/* '><textarea> */ data-value={42}",
            "{...props}/* '><textarea> */",
            "/* '><textarea> */ title=\"value\" // '><textarea>\n data-value={42}",
            "title=\"/* '><textarea> */\"",
            "title=\"// '><textarea>\"",
            "data-url=https://example.com/path",
            "data-value=foo/*bar*/suffix",
            "data-value = //literal",
            "data-value = /*literal*/suffix",
            "data-value={value}//literal",
            "data-value={value}/*literal*/suffix",
            "data-value=日/*literal*/suffix",
            "data-url=https://example.com/path /* '><textarea> */",
        ] {
            let source_text = format!("<div {attributes}></div><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_script_after_declaration_tags() {
        for markup in [
            "{const x = '<textarea>'}<p>{x}</p>",
            "{let x = '<textarea>'}<p>{x}</p>",
            "{const {x = '<textarea>'} = value}<p>{x}</p>",
            "{let x: string = '<textarea>', y = '<style>'}<p>{x}</p>",
            "{const x = await Promise.resolve('<textarea>')}<p>{x}</p>",
            "{const x = `text ${value as string} <script>ignored</script>`}<p>{x}</p>",
            "{let x = /[}]/.test(value) ? '<textarea>' : ''}<p>{x}</p>",
            "{const x = ( // } <script>ignored</script>\n'<textarea>')}<p>{x}</p>",
            "{let x = '<textarea>';}<p>{x}</p>",
            "{const/* } <script>ignored</script> */x = '<textarea>'}<p>{x}</p>",
            "{constant('<textarea>')}<p>hi</p>",
            "{letter('<textarea>')}<p>hi</p>",
        ] {
            let source_text = format!("{markup}<script lang='ts'>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_script_after_svelte_binding_headers() {
        for markup in [
            "{#await promise then}<p>done</p>{/await}",
            "{#await promise catch }<p>failed</p>{/await}",
            "{#await promise}{:then {x = '<textarea>'}}<p>done</p>{:catch {message = '<textarea>'}}<p>failed</p>{/await}",
            "{@const {x = '<textarea>'} = value}",
            "{#if true}{@const x = await Promise.resolve('<textarea>')}{/if}",
            "{#each items as {x = await Promise.resolve('<textarea>')}}{/each}",
            "{#await p then {x = await Promise.resolve('<textarea>')}}{/await}",
            "{#await p}{:then {x = await Promise.resolve('<textarea>')}}{/await}",
            "{#each (items)as {x = '<textarea>'}}<p>hi</p>{/each}",
            "{#snippet content<T>(x: T)}<p>hi</p>{/snippet}",
        ] {
            let source_text = format!("{markup}<script lang='ts'>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_script_after_large_header_literals_and_comments() {
        let text = "as } ( ".repeat(4096);
        for markup in [
            format!("{{#each fn('{text}') as item}}<p>hi</p>{{/each}}"),
            format!(
                "{{#each fn((x: string) => x) as {{x /* {text} */ = '<textarea>'}}}}<p>hi</p>{{/each}}"
            ),
            format!("{{#each items as {{'{text}': x = '<textarea>'}}, i (i)}}<p>hi</p>{{/each}}"),
        ] {
            let source_text = format!("{markup}<script lang='ts'>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].source_text, "debugger;");
        }
    }

    #[test]
    fn test_expression_parsing_chunk_boundaries() {
        for len in [120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 255, 256, 257] {
            for expression in [
                format!("'{}<textarea>'", "a".repeat(len)),
                format!("'{}<textarea>'", "日".repeat(len)),
                format!(r"/{}}}<textarea>/.source", "a".repeat(len)),
                format!("value {} / 2", " ".repeat(len)),
                format!("value {} // <textarea> }}\n", " ".repeat(len)),
                format!("value /* {} <textarea> }} */", "a".repeat(len)),
                format!("`{}${{value as string}}<textarea>`", "a".repeat(len)),
                format!("`a${{value + {} other}}<textarea>`", " ".repeat(len)),
            ] {
                let source_text = format!("{{@html {expression}}}<script>debugger;</script>");
                let sources = SveltePartialLoader::new(&source_text).parse();
                assert_eq!(sources.len(), 1, "{len}: {expression}");
                assert_eq!(sources[0].source_text, "debugger;");
            }
        }
    }
    #[test]
    fn test_component_attribute_context_after_markup() {
        for prefix in [
            "<br>",
            "<command><keygen>",
            "<div></div>",
            "<div\u{feff}></div>",
            "<p>text<hr>",
            "<p><hr/>",
            "<p><div/>",
            "{#if ok}<p>one{:else}<p>two{/if}",
            "<Widget />",
            "<svelte:head><title>x</title></svelte:head>",
        ] {
            let source_text = format!(
                "{prefix}<style title=\"before {{\">.a {{ color: red; }}</style><script>debugger;</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_textarea_closing_tag_variants() {
        for closing in
            ["</TEXTAREA>", "</TextArea>", "</textarea\u{feff}>", "</textarea title=\">\">"]
        {
            let source_text = format!("<textarea>text{closing}<script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_textarea_expression_with_closing_tag_text() {
        let source_text = r"<textarea>{'</textarea><script>ignored</script>'}</textarea><script>debugger;</script>";
        let sources = SveltePartialLoader::new(source_text).parse();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].source_text, "debugger;");
    }

    #[test]
    fn test_preprocessed_styles_preserve_scripts() {
        for lang in ["scss", "sass", "less", "stylus"] {
            for comment in ["// {", "// (", "// \"", "// </stylex>"] {
                let source_text = format!(
                    "<style lang=\"{lang}\">{comment}\n.a {{ color: red; }}\n</style ><script>debugger;</script>"
                );
                let sources = SveltePartialLoader::new(&source_text).parse();
                assert_eq!(sources.len(), 1, "{source_text}");
                assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
            }
        }
    }

    #[test]
    fn test_self_closing_scripts_preserve_component_scripts() {
        for element in ["<script/>", "<script />", "<script/\u{feff}>", "<script title={\"}\"}/>"] {
            let source_text =
                format!("<svelte:head>{element}</svelte:head><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_component_style_with_url_delimiters() {
        for url in ["foo{", "foo[", "foo/*bar", "foo<!--bar", r"foo\)bar{", "'foo){'"] {
            let source_text = format!(
                "<style>.a {{ background: url({url}); }}</style><script>debugger;</script>"
            );
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_component_style_with_closing_tag_text() {
        for css in [
            r#".a::before { content: "</style><script>ignored</script>"; }"#,
            ".a { /* </style><script>ignored</script> */ color: red; }",
            "/* </style><script>ignored</script> */ .a { color: red; }",
            ".a { background: url(var(--x)) </style><script>ignored</script>; }",
            "<!-- { --> .a { color: red; }",
            "<!-- </style><script>ignored</script> --> .a { color: red; }",
        ] {
            let source_text = format!("<style>{css}</style><script>debugger;</script>");
            let sources = SveltePartialLoader::new(&source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert_eq!(sources[0].source_text, "debugger;", "{source_text}");
        }
    }

    #[test]
    fn test_style_attribute_contexts_preserve_scripts() {
        for source_text in [
            r#"<style data-x={"}"}>.a { color: red; }</style><script>const text = `">`; debugger;</script>"#,
            r#"<svelte:head><style title={"}"}>.a { color: red; }</style></svelte:head><script>const text = `">`; debugger;</script>"#,
            r#"<div><style title="before {"></style><script>ignored</script>"} after">.a { color: red; }</style></div><script>debugger;</script>"#,
        ] {
            let sources = SveltePartialLoader::new(source_text).parse();
            assert_eq!(sources.len(), 1, "{source_text}");
            assert!(sources[0].source_text.contains("debugger;"), "{source_text}");
        }
    }
}

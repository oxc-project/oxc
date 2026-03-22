use std::{borrow::Cow, collections::HashMap, path::Path};

use phf::phf_set;

use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter::{FormatOptions, IndentStyle, LineEnding, QuoteProperties, QuoteStyle};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum JsonFlavor {
    Json,
    Jsonc,
    Json5,
    JsonStringify,
}

impl JsonFlavor {
    pub(crate) fn from_path(path: &Path) -> Option<Self> {
        let file_name = path.file_name().and_then(|f| f.to_str())?;
        let extension = path.extension().and_then(|ext| ext.to_str());

        if file_name == "package.json"
            || file_name == "composer.json"
            || extension == Some("importmap")
        {
            return Some(Self::JsonStringify);
        }

        if JSON_FILENAMES.contains(file_name) {
            return Some(Self::Json);
        }
        if let Some(ext) = extension
            && JSON_EXTENSIONS.contains(ext)
        {
            return Some(Self::Json);
        }
        if let Some(ext) = extension
            && JSONC_EXTENSIONS.contains(ext)
        {
            return Some(Self::Jsonc);
        }

        match extension {
            Some("json") => Some(Self::Json),
            Some("jsonc") => Some(Self::Jsonc),
            Some("json5") => Some(Self::Json5),
            _ => None,
        }
    }

    fn allows_comments(self) -> bool {
        !matches!(self, Self::JsonStringify)
    }

    fn allows_comment_only_input(self) -> bool {
        matches!(self, Self::Jsonc)
    }

    fn allows_single_quotes(self) -> bool {
        matches!(self, Self::Json | Self::Jsonc | Self::Json5 | Self::JsonStringify)
    }

    fn allows_unquoted_keys(self) -> bool {
        matches!(self, Self::Json | Self::Jsonc | Self::Json5 | Self::JsonStringify)
    }

    fn allows_trailing_commas_in_input(self) -> bool {
        true
    }

    fn allows_extended_scalars(self) -> bool {
        matches!(self, Self::Json | Self::Jsonc | Self::Json5 | Self::JsonStringify)
    }

    fn prints_trailing_commas(self, options: &FormatOptions) -> bool {
        matches!(self, Self::Jsonc | Self::Json5) && !options.trailing_commas.is_none()
    }

    fn prints_comments(self) -> bool {
        !matches!(self, Self::JsonStringify)
    }

    fn forces_expanded_layout(self) -> bool {
        matches!(self, Self::JsonStringify)
    }
}

static JSON_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "json",
    "4DForm",
    "4DProject",
    "avsc",
    "geojson",
    "gltf",
    "har",
    "ice",
    "JSON-tmLanguage",
    "json.example",
    "mcmeta",
    "sarif",
    "tact",
    "tfstate",
    "tfstate.backup",
    "topojson",
    "webapp",
    "webmanifest",
    "yy",
    "yyp",
};

static JSON_FILENAMES: phf::Set<&'static str> = phf_set! {
    ".all-contributorsrc",
    ".arcconfig",
    ".auto-changelog",
    ".babelrc",
    ".c8rc",
    ".htmlhintrc",
    ".imgbotconfig",
    ".jscsrc",
    ".jshintrc",
    ".jslintrc",
    ".nycrc",
    ".swcrc",
    ".tern-config",
    ".tern-project",
    ".watchmanconfig",
};

static JSONC_EXTENSIONS: phf::Set<&'static str> = phf_set! {
    "code-snippets",
    "code-workspace",
    "jsonc",
    "sublime-build",
    "sublime-color-scheme",
    "sublime-commands",
    "sublime-completions",
    "sublime-keymap",
    "sublime-macro",
    "sublime-menu",
    "sublime-mousemap",
    "sublime-project",
    "sublime-settings",
    "sublime-theme",
    "sublime-workspace",
    "sublime_metrics",
    "sublime_session",
};

pub(crate) fn format_json(
    source_text: &str,
    path: &Path,
    flavor: JsonFlavor,
    options: &FormatOptions,
) -> Result<String, OxcDiagnostic> {
    let document = Parser::new(source_text, flavor).parse_document().map_err(|err| {
        OxcDiagnostic::error(format!("Failed to parse {} as {:?}: {err}", path.display(), flavor))
    })?;

    let mut output = String::with_capacity(source_text.len().saturating_add(source_text.len() / 4));
    {
        let mut printer = Printer::new(options, flavor, &mut output);
        printer.print_document(&document);
        if !printer.ends_with_line_ending() {
            printer.push_line_ending();
        }
    }

    Ok(output)
}

#[derive(Debug)]
struct Document<'a> {
    leading_comments: Vec<Comment<'a>>,
    root: Option<Node<'a>>,
    trailing_comments: Vec<Comment<'a>>,
}

#[derive(Debug)]
struct Node<'a> {
    leading_comments: Vec<Comment<'a>>,
    kind: NodeKind<'a>,
    trailing_block_comments: Vec<Comment<'a>>,
}

#[derive(Debug)]
enum NodeKind<'a> {
    Object(ObjectNode<'a>),
    Array(ArrayNode<'a>),
    Scalar(Scalar<'a>),
}

#[derive(Debug)]
enum Scalar<'a> {
    String(StringValue<'a>),
    Number(&'a str),
    Boolean(bool),
    Null,
    Identifier(&'a str),
}

#[derive(Debug)]
struct StringValue<'a> {
    value: Cow<'a, str>,
    preferred_quote: QuoteStyle,
}

#[derive(Debug)]
struct ObjectNode<'a> {
    members: Vec<Member<'a>>,
    trailing_comments: Vec<Comment<'a>>,
}

#[derive(Debug)]
struct Member<'a> {
    leading_comments: Vec<Comment<'a>>,
    key: Key<'a>,
    value: Node<'a>,
}

#[derive(Debug)]
enum Key<'a> {
    String { value: Cow<'a, str>, preferred_quote: QuoteStyle, preserve_quote: bool },
    Identifier(&'a str),
    Number(&'a str),
}

#[derive(Debug)]
struct ArrayNode<'a> {
    elements: Vec<Node<'a>>,
    trailing_comments: Vec<Comment<'a>>,
}

#[derive(Debug, Clone, Copy)]
struct Comment<'a> {
    raw: &'a str,
    has_newline: bool,
}

struct Parser<'a> {
    source_text: &'a str,
    bytes: &'a [u8],
    flavor: JsonFlavor,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source_text: &'a str, flavor: JsonFlavor) -> Self {
        Self { source_text, bytes: source_text.as_bytes(), flavor, pos: 0 }
    }

    fn parse_document(&mut self) -> Result<Document<'a>, String> {
        let leading_comments = self.parse_leading_comments()?;
        if self.is_eof() {
            if leading_comments.is_empty() {
                return Ok(Document { leading_comments, root: None, trailing_comments: vec![] });
            }
            if self.flavor.allows_comment_only_input() {
                return Ok(Document {
                    leading_comments: vec![],
                    root: None,
                    trailing_comments: leading_comments,
                });
            }
            return Err("The input is empty or contains only comments".to_string());
        }

        let root = Some(self.parse_node_with_leading_comments(leading_comments)?);
        let trailing_comments = self.parse_leading_comments()?;
        self.skip_whitespace();
        if !self.is_eof() {
            return Err(format!("Unexpected token at byte {}", self.pos));
        }
        Ok(Document { leading_comments: vec![], root, trailing_comments })
    }

    fn parse_node(&mut self) -> Result<Node<'a>, String> {
        let leading_comments = self.parse_leading_comments()?;
        self.parse_node_with_leading_comments(leading_comments)
    }

    fn parse_node_with_leading_comments(
        &mut self,
        leading_comments: Vec<Comment<'a>>,
    ) -> Result<Node<'a>, String> {
        let kind = self.parse_node_kind()?;
        let trailing_block_comments = self.parse_trailing_block_comments()?;
        Ok(Node { leading_comments, kind, trailing_block_comments })
    }

    fn parse_node_kind(&mut self) -> Result<NodeKind<'a>, String> {
        match self.peek_byte() {
            Some(b'{') => self.parse_object().map(NodeKind::Object),
            Some(b'[') => self.parse_array().map(NodeKind::Array),
            Some(b'"') => {
                self.parse_string(b'"').map(|value| NodeKind::Scalar(Scalar::String(value)))
            }
            Some(b'\'') if self.flavor.allows_single_quotes() => {
                self.parse_string(b'\'').map(|value| NodeKind::Scalar(Scalar::String(value)))
            }
            Some(b'`') if self.flavor.allows_single_quotes() => {
                self.parse_template_string().map(|value| NodeKind::Scalar(Scalar::String(value)))
            }
            Some(b'+') | Some(b'-') => self.parse_signed_scalar().map(NodeKind::Scalar),
            Some(_) => self.parse_scalar().map(NodeKind::Scalar),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<ObjectNode<'a>, String> {
        self.expect_byte(b'{')?;
        let mut members = Vec::new();
        let mut trailing_comments = Vec::new();
        let mut saw_trailing_comma = false;

        loop {
            let leading_comments = self.parse_leading_comments()?;
            if self.peek_byte() == Some(b'}') {
                trailing_comments = leading_comments;
                self.pos += 1;
                break;
            }

            let key = self.parse_key()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_node()?;
            members.push(Member { leading_comments, key, value });

            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.pos += 1;
                    saw_trailing_comma = true;
                }
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => return Err(format!("Expected ',' or '}}' at byte {}", self.pos)),
                None => return Err("Unexpected end of input inside object".to_string()),
            }
        }

        if saw_trailing_comma && !self.flavor.allows_trailing_commas_in_input() {
            return Err("Trailing commas are not allowed in this JSON mode".to_string());
        }

        Ok(ObjectNode { members, trailing_comments })
    }

    fn parse_array(&mut self) -> Result<ArrayNode<'a>, String> {
        self.expect_byte(b'[')?;
        let mut elements = Vec::new();
        let mut trailing_comments = Vec::new();
        let mut saw_trailing_comma = false;

        loop {
            let leading_comments = self.parse_leading_comments()?;
            if self.peek_byte() == Some(b']') {
                trailing_comments = leading_comments;
                self.pos += 1;
                break;
            }

            let node = self.parse_node_with_leading_comments(leading_comments)?;
            elements.push(node);

            self.skip_whitespace();
            match self.peek_byte() {
                Some(b',') => {
                    self.pos += 1;
                    saw_trailing_comma = true;
                }
                Some(b']') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => return Err(format!("Expected ',' or ']' at byte {}", self.pos)),
                None => return Err("Unexpected end of input inside array".to_string()),
            }
        }

        if saw_trailing_comma && !self.flavor.allows_trailing_commas_in_input() {
            return Err("Trailing commas are not allowed in this JSON mode".to_string());
        }

        Ok(ArrayNode { elements, trailing_comments })
    }

    fn parse_key(&mut self) -> Result<Key<'a>, String> {
        match self.peek_byte() {
            Some(b'"') => self.parse_key_string(b'"'),
            Some(b'\'') if self.flavor.allows_single_quotes() => self.parse_key_string(b'\''),
            Some(b'`') if self.flavor.allows_single_quotes() => self.parse_key_template(),
            Some(_) if self.flavor.allows_unquoted_keys() => self.parse_unquoted_key(),
            Some(_) => Err(format!("Expected quoted object key at byte {}", self.pos)),
            None => Err("Unexpected end of input while parsing key".to_string()),
        }
    }

    fn parse_key_string(&mut self, quote: u8) -> Result<Key<'a>, String> {
        let value = self.parse_string(quote)?;
        Ok(Key::String {
            preserve_quote: true,
            preferred_quote: value.preferred_quote,
            value: value.value,
        })
    }

    fn parse_key_template(&mut self) -> Result<Key<'a>, String> {
        let value = self.parse_template_string()?;
        Ok(Key::String {
            preserve_quote: true,
            preferred_quote: value.preferred_quote,
            value: value.value,
        })
    }

    fn parse_unquoted_key(&mut self) -> Result<Key<'a>, String> {
        let start = self.pos;
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_whitespace() || matches!(byte, b':' | b',' | b'}' | b']') {
                break;
            }
            if byte == b'/' && self.flavor.allows_comments() {
                break;
            }
            self.pos += 1;
        }
        if start == self.pos {
            return Err(format!("Expected key at byte {}", self.pos));
        }

        let raw = &self.source_text[start..self.pos];
        if is_identifier_like(raw) {
            return Ok(Key::Identifier(raw));
        }
        if self.is_valid_number(raw) {
            return Ok(Key::Number(raw));
        }
        Err(format!("Unsupported object key `{raw}`"))
    }

    fn parse_signed_scalar(&mut self) -> Result<Scalar<'a>, String> {
        let start = self.pos;
        let sign = self.peek_byte().expect("sign must exist");
        self.pos += 1;
        match self.peek_byte() {
            Some(b'"') => {
                Err(format!("Operator '{}' before string literal is not allowed", char::from(sign)))
            }
            Some(b'\'') | Some(b'`') => {
                Err(format!("Operator '{}' before string literal is not allowed", char::from(sign)))
            }
            _ => {
                let value = self.parse_scalar_token(start)?;
                match value {
                    Scalar::Number(_) => Ok(value),
                    Scalar::Identifier(name) if matches!(name, "Infinity" | "NaN") => {
                        Ok(Scalar::Identifier(&self.source_text[start..self.pos]))
                    }
                    _ => Err(format!(
                        "Unsupported signed scalar `{}`",
                        &self.source_text[start..self.pos]
                    )),
                }
            }
        }
    }

    fn parse_scalar(&mut self) -> Result<Scalar<'a>, String> {
        let start = self.pos;
        self.parse_scalar_token(start)
    }

    fn parse_scalar_token(&mut self, start: usize) -> Result<Scalar<'a>, String> {
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_whitespace() || matches!(byte, b',' | b']' | b'}') {
                break;
            }
            if byte == b'/' && self.flavor.allows_comments() {
                break;
            }
            self.pos += 1;
        }

        if start == self.pos {
            return Err(format!("Expected scalar value at byte {}", self.pos));
        }

        let raw = &self.source_text[start..self.pos];
        match raw {
            "true" => Ok(Scalar::Boolean(true)),
            "false" => Ok(Scalar::Boolean(false)),
            "null" => Ok(Scalar::Null),
            _ if self.is_valid_number(raw) => Ok(Scalar::Number(raw)),
            _ if self.is_valid_extended_scalar(raw) => Ok(Scalar::Identifier(raw)),
            _ => Err(format!("Unsupported scalar value `{raw}`")),
        }
    }

    fn parse_string(&mut self, quote: u8) -> Result<StringValue<'a>, String> {
        let preferred_quote = QuoteStyle::from_byte(quote)
            .ok_or_else(|| "Internal quote conversion error".to_string())?;
        self.pos += 1;
        let content_start = self.pos;
        let mut chunk_start = self.pos;
        let mut owned: Option<String> = None;

        while let Some(byte) = self.peek_byte() {
            match byte {
                b'\n' | b'\r' => return Err("Unterminated string literal".to_string()),
                b'\\' => {
                    if let Some(buffer) = owned.as_mut() {
                        buffer.push_str(&self.source_text[chunk_start..self.pos]);
                    } else {
                        owned = Some(self.source_text[content_start..self.pos].to_string());
                    }
                    self.pos += 1;
                    let decoded = self.parse_escape(quote)?;
                    owned.as_mut().expect("owned string must exist after escape").push(decoded);
                    chunk_start = self.pos;
                }
                _ if byte == quote => {
                    let value = if let Some(mut buffer) = owned {
                        buffer.push_str(&self.source_text[chunk_start..self.pos]);
                        Cow::Owned(buffer)
                    } else {
                        Cow::Borrowed(&self.source_text[content_start..self.pos])
                    };
                    self.pos += 1;
                    return Ok(StringValue { value, preferred_quote });
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        Err("Unterminated string literal".to_string())
    }

    fn parse_template_string(&mut self) -> Result<StringValue<'a>, String> {
        self.pos += 1;
        let content_start = self.pos;
        let mut chunk_start = self.pos;
        let mut owned: Option<String> = None;

        while let Some(byte) = self.peek_byte() {
            match byte {
                b'`' => {
                    let value = if let Some(mut buffer) = owned {
                        buffer.push_str(&self.source_text[chunk_start..self.pos]);
                        Cow::Owned(buffer)
                    } else {
                        Cow::Borrowed(&self.source_text[content_start..self.pos])
                    };
                    self.pos += 1;
                    return Ok(StringValue { value, preferred_quote: QuoteStyle::Double });
                }
                b'\\' => {
                    if let Some(buffer) = owned.as_mut() {
                        buffer.push_str(&self.source_text[chunk_start..self.pos]);
                    } else {
                        owned = Some(self.source_text[content_start..self.pos].to_string());
                    }
                    self.pos += 1;
                    let decoded = self.parse_escape(b'`')?;
                    owned.as_mut().expect("owned string must exist after escape").push(decoded);
                    chunk_start = self.pos;
                }
                b'$' if self.peek_next_byte() == Some(b'{') => {
                    return Err(
                        "Template literals with expressions are not allowed in JSON".to_string()
                    );
                }
                _ => {
                    self.pos += 1;
                }
            }
        }

        Err("Unterminated template literal".to_string())
    }

    fn parse_escape(&mut self, quote: u8) -> Result<char, String> {
        let byte = self.peek_byte().ok_or_else(|| "Incomplete escape sequence".to_string())?;
        self.pos += 1;
        match byte {
            b'"' => Ok('"'),
            b'\'' => Ok('\''),
            b'`' if quote == b'`' => Ok('`'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000C}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'v' => Ok('\u{000B}'),
            b'0' => Ok('\0'),
            b'x' => {
                let hi = self.take_hex_digit()?;
                let lo = self.take_hex_digit()?;
                char::from_u32((hi << 4) | lo).ok_or_else(|| "Invalid hex escape".to_string())
            }
            b'u' => {
                let mut code = 0_u32;
                for _ in 0..4 {
                    code = (code << 4) | self.take_hex_digit()?;
                }
                char::from_u32(code).ok_or_else(|| "Invalid unicode escape".to_string())
            }
            other => Ok(other as char),
        }
    }

    fn take_hex_digit(&mut self) -> Result<u32, String> {
        let byte = self.peek_byte().ok_or_else(|| "Incomplete hex escape".to_string())?;
        self.pos += 1;
        match byte {
            b'0'..=b'9' => Ok(u32::from(byte - b'0')),
            b'a'..=b'f' => Ok(u32::from(byte - b'a' + 10)),
            b'A'..=b'F' => Ok(u32::from(byte - b'A' + 10)),
            _ => Err("Invalid hex escape".to_string()),
        }
    }

    fn parse_trailing_block_comments(&mut self) -> Result<Vec<Comment<'a>>, String> {
        let mut comments = Vec::new();
        loop {
            self.skip_inline_whitespace();
            if self.starts_with_bytes(b"/*") {
                comments.push(self.parse_comment()?);
            } else {
                break;
            }
        }
        Ok(comments)
    }

    fn parse_leading_comments(&mut self) -> Result<Vec<Comment<'a>>, String> {
        let mut comments = Vec::new();
        loop {
            self.skip_whitespace();
            if self.starts_with_bytes(b"//") || self.starts_with_bytes(b"/*") {
                if !self.flavor.allows_comments() {
                    return Err("Comment is not allowed in JSON".to_string());
                }
                comments.push(self.parse_comment()?);
            } else {
                break;
            }
        }
        Ok(comments)
    }

    fn parse_comment(&mut self) -> Result<Comment<'a>, String> {
        if self.starts_with_bytes(b"//") {
            let start = self.pos;
            self.pos += 2;
            while let Some(byte) = self.peek_byte() {
                if matches!(byte, b'\n' | b'\r') {
                    break;
                }
                self.pos += 1;
            }
            return Ok(Comment { raw: &self.source_text[start..self.pos], has_newline: false });
        }

        if self.starts_with_bytes(b"/*") {
            let start = self.pos;
            self.pos += 2;
            while self.pos + 1 < self.bytes.len() {
                if self.starts_with_bytes(b"*/") {
                    self.pos += 2;
                    let raw = &self.source_text[start..self.pos];
                    return Ok(Comment { raw, has_newline: raw.as_bytes().contains(&b'\n') });
                }
                self.pos += 1;
            }
            return Err("Unterminated block comment".to_string());
        }

        Err(format!("Expected comment at byte {}", self.pos))
    }

    fn is_valid_number(&self, raw: &str) -> bool {
        let bytes = raw.as_bytes();
        if bytes.is_empty() {
            return false;
        }

        let mut index = 0;
        if matches!(bytes[index], b'+' | b'-') {
            index += 1;
            if index == bytes.len() {
                return false;
            }
        }

        if index + 1 < bytes.len()
            && bytes[index] == b'0'
            && matches!(bytes[index + 1], b'x' | b'X')
        {
            index += 2;
            return index < bytes.len() && bytes[index..].iter().all(u8::is_ascii_hexdigit);
        }

        if bytes[index] == b'0' {
            index += 1;
        } else if bytes[index].is_ascii_digit() {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
        } else {
            return false;
        }

        if index < bytes.len() && bytes[index] == b'.' {
            index += 1;
            if index == bytes.len() || !bytes[index].is_ascii_digit() {
                return false;
            }
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
        }

        if index < bytes.len() && matches!(bytes[index], b'e' | b'E') {
            index += 1;
            if index < bytes.len() && matches!(bytes[index], b'+' | b'-') {
                index += 1;
            }
            if index == bytes.len() || !bytes[index].is_ascii_digit() {
                return false;
            }
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
        }

        index == bytes.len()
    }

    fn is_valid_extended_scalar(&self, raw: &str) -> bool {
        if !self.flavor.allows_extended_scalars() {
            return false;
        }

        matches!(
            raw,
            "Infinity" | "-Infinity" | "+Infinity" | "NaN" | "+NaN" | "-NaN" | "undefined"
        ) || is_identifier_like(raw)
    }

    fn expect_byte(&mut self, byte: u8) -> Result<(), String> {
        match self.peek_byte() {
            Some(found) if found == byte => {
                self.pos += 1;
                Ok(())
            }
            Some(found) => Err(format!(
                "Expected `{}` at byte {}, found `{}`",
                char::from(byte),
                self.pos,
                char::from(found)
            )),
            None => Err(format!("Expected `{}` but found end of input", char::from(byte))),
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn skip_inline_whitespace(&mut self) {
        while let Some(byte) = self.peek_byte() {
            if matches!(byte, b' ' | b'\t') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn starts_with_bytes(&self, needle: &[u8]) -> bool {
        self.bytes.get(self.pos..self.pos + needle.len()) == Some(needle)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_next_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}

fn is_identifier_like(raw: &str) -> bool {
    let mut chars = raw.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || matches!(first, '_' | '$'))
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$'))
}

struct Printer<'a, 'b> {
    options: &'a FormatOptions,
    output: &'b mut String,
    flavor: JsonFlavor,
    line_ending: &'static str,
    indent_unit: IndentUnit,
    allow_trailing_comma: bool,
    print_width: usize,
    node_width_cache: HashMap<usize, Option<usize>>,
    object_width_cache: HashMap<usize, Option<usize>>,
    array_width_cache: HashMap<usize, Option<usize>>,
    object_consistent_cache: HashMap<usize, bool>,
}

#[derive(Clone, Copy)]
enum IndentUnit {
    Tab,
    Space(usize),
}

impl<'a, 'b> Printer<'a, 'b> {
    fn new(options: &'a FormatOptions, flavor: JsonFlavor, output: &'b mut String) -> Self {
        let line_ending = match options.line_ending {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        };

        let indent_unit = match options.indent_style {
            IndentStyle::Tab => IndentUnit::Tab,
            IndentStyle::Space => IndentUnit::Space(usize::from(options.indent_width.value())),
        };

        Self {
            options,
            output,
            flavor,
            line_ending,
            indent_unit,
            allow_trailing_comma: flavor.prints_trailing_commas(options),
            print_width: usize::from(options.line_width.value()),
            node_width_cache: HashMap::new(),
            object_width_cache: HashMap::new(),
            array_width_cache: HashMap::new(),
            object_consistent_cache: HashMap::new(),
        }
    }

    fn print_document(&mut self, document: &Document<'_>) {
        if let Some(root) = &document.root {
            self.write_comments(&document.leading_comments, 0);
            self.write_node(root, 0);
            if !document.trailing_comments.is_empty() {
                self.push_line_ending();
                self.write_comments(&document.trailing_comments, 0);
            }
        } else {
            self.write_comments(&document.trailing_comments, 0);
        }
    }

    fn write_node(&mut self, node: &Node<'_>, indent: usize) {
        self.write_comments(&node.leading_comments, indent);
        match &node.kind {
            NodeKind::Scalar(scalar) => self.write_scalar(scalar),
            NodeKind::Object(object) => self.write_object(object, indent),
            NodeKind::Array(array) => self.write_array(array, indent),
        }
        if self.flavor.prints_comments() {
            for comment in &node.trailing_block_comments {
                self.output.push(' ');
                self.output.push_str(comment.raw);
            }
        }
    }

    fn write_scalar(&mut self, scalar: &Scalar<'_>) {
        match scalar {
            Scalar::String(value) => self.write_string_value(value),
            Scalar::Number(raw) | Scalar::Identifier(raw) => self.output.push_str(raw),
            Scalar::Boolean(value) => self.output.push_str(if *value { "true" } else { "false" }),
            Scalar::Null => self.output.push_str("null"),
        }
    }

    fn write_object(&mut self, object: &ObjectNode<'_>, indent: usize) {
        if !self.flavor.forces_expanded_layout()
            && self
                .object_exact_inline_width(object)
                .is_some_and(|width| width <= self.line_width_at_indent(indent))
        {
            self.write_object_inline(object);
            return;
        }

        self.output.push('{');
        if object.members.is_empty() && object.trailing_comments.is_empty() {
            self.output.push('}');
            return;
        }
        self.push_line_ending();

        let consistent = self.object_quotes_consistent(object);

        for (index, member) in object.members.iter().enumerate() {
            self.write_comments(&member.leading_comments, indent + 1);
            self.write_indent(indent + 1);
            self.write_key(&member.key, consistent);
            self.output.push(':');

            if self
                .node_inline_width(&member.value, self.line_width_at_indent(indent + 1))
                .is_some()
                && member.value.leading_comments.is_empty()
            {
                self.output.push(' ');
                self.write_node_inline(&member.value);
            } else {
                self.push_line_ending();
                self.write_node(&member.value, indent + 1);
            }

            if index + 1 < object.members.len()
                || (self.allow_trailing_comma
                    && object.trailing_comments.is_empty()
                    && self.flavor.prints_comments())
            {
                self.output.push(',');
            }
            self.push_line_ending();
        }

        self.write_comments(&object.trailing_comments, indent + 1);
        self.write_indent(indent);
        self.output.push('}');
    }

    fn write_array(&mut self, array: &ArrayNode<'_>, indent: usize) {
        if !self.flavor.forces_expanded_layout()
            && self
                .array_exact_inline_width(array)
                .is_some_and(|width| width <= self.line_width_at_indent(indent))
        {
            self.write_array_inline(array);
            return;
        }

        self.output.push('[');
        if array.elements.is_empty() && array.trailing_comments.is_empty() {
            self.output.push(']');
            return;
        }
        self.push_line_ending();

        for (index, element) in array.elements.iter().enumerate() {
            self.write_indent(indent + 1);
            self.write_node(element, indent + 1);
            if index + 1 < array.elements.len()
                || (self.allow_trailing_comma
                    && array.trailing_comments.is_empty()
                    && self.flavor.prints_comments())
            {
                self.output.push(',');
            }
            self.push_line_ending();
        }

        self.write_comments(&array.trailing_comments, indent + 1);
        self.write_indent(indent);
        self.output.push(']');
    }

    fn write_node_inline(&mut self, node: &Node<'_>) {
        match &node.kind {
            NodeKind::Scalar(scalar) => self.write_scalar(scalar),
            NodeKind::Object(object) => self.write_object_inline(object),
            NodeKind::Array(array) => self.write_array_inline(array),
        }
    }

    fn write_object_inline(&mut self, object: &ObjectNode<'_>) {
        self.output.push('{');
        if object.members.is_empty() {
            self.output.push('}');
            return;
        }

        let consistent = self.object_quotes_consistent(object);

        if self.options.bracket_spacing.value() {
            self.output.push(' ');
        }

        for (index, member) in object.members.iter().enumerate() {
            self.write_key(&member.key, consistent);
            self.output.push(':');
            self.output.push(' ');
            self.write_node_inline(&member.value);
            if index + 1 < object.members.len() {
                self.output.push(',');
                self.output.push(' ');
            }
        }

        if self.options.bracket_spacing.value() {
            self.output.push(' ');
        }
        self.output.push('}');
    }

    fn write_array_inline(&mut self, array: &ArrayNode<'_>) {
        self.output.push('[');
        for (index, element) in array.elements.iter().enumerate() {
            self.write_node_inline(element);
            if index + 1 < array.elements.len() {
                self.output.push(',');
                self.output.push(' ');
            }
        }
        self.output.push(']');
    }

    fn write_key(&mut self, key: &Key<'_>, consistent: bool) {
        match key {
            Key::Identifier(name) => {
                if self.should_quote_key(key, consistent) {
                    self.write_string(name, self.key_quote_style());
                } else {
                    self.output.push_str(name);
                }
            }
            Key::Number(raw) => {
                if self.should_quote_key(key, consistent) {
                    self.write_string(raw, self.key_quote_style());
                } else {
                    self.output.push_str(raw);
                }
            }
            Key::String { value, preferred_quote, .. } => {
                if self.should_quote_key(key, consistent) {
                    self.write_string(
                        value,
                        self.key_quote_style_for_string(*preferred_quote, key),
                    );
                } else {
                    self.output.push_str(value);
                }
            }
        }
    }

    fn should_quote_key(&self, key: &Key<'_>, consistent: bool) -> bool {
        match self.flavor {
            JsonFlavor::Json | JsonFlavor::Jsonc | JsonFlavor::JsonStringify => true,
            JsonFlavor::Json5 => match key {
                Key::Number(_) => false,
                Key::Identifier(name) => consistent || !is_identifier_like(name),
                Key::String { value, preserve_quote, .. } => {
                    consistent
                        || !is_identifier_like(value)
                        || (*preserve_quote
                            && matches!(self.options.quote_properties, QuoteProperties::Preserve))
                }
            },
        }
    }

    fn write_string_value(&mut self, value: &StringValue<'_>) {
        let quote = match self.flavor {
            JsonFlavor::Json | JsonFlavor::Jsonc | JsonFlavor::JsonStringify => QuoteStyle::Double,
            JsonFlavor::Json5 => self.options.quote_style,
        };
        let _ = value.preferred_quote;
        self.write_string(&value.value, quote);
    }

    fn key_quote_style(&self) -> QuoteStyle {
        match self.flavor {
            JsonFlavor::Json | JsonFlavor::Jsonc | JsonFlavor::JsonStringify => QuoteStyle::Double,
            JsonFlavor::Json5 => self.options.quote_style,
        }
    }

    fn key_quote_style_for_string(
        &self,
        _preferred_quote: QuoteStyle,
        _key: &Key<'_>,
    ) -> QuoteStyle {
        match self.flavor {
            JsonFlavor::Json | JsonFlavor::Jsonc | JsonFlavor::JsonStringify => QuoteStyle::Double,
            JsonFlavor::Json5 => self.options.quote_style,
        }
    }

    fn write_string(&mut self, value: &str, quote: QuoteStyle) {
        let quote_char = quote.as_char();
        self.output.push(quote_char);
        if !string_needs_escaping(value, quote) {
            self.output.push_str(value);
            self.output.push(quote_char);
            return;
        }
        for ch in value.chars() {
            match ch {
                '\\' => self.output.push_str("\\\\"),
                '"' if quote.is_double() => self.output.push_str("\\\""),
                '\'' if !quote.is_double() => self.output.push_str("\\'"),
                '\u{0008}' => self.output.push_str("\\b"),
                '\u{000C}' => self.output.push_str("\\f"),
                '\n' => self.output.push_str("\\n"),
                '\r' => self.output.push_str("\\r"),
                '\t' => self.output.push_str("\\t"),
                c if c < ' ' => {
                    use std::fmt::Write as _;
                    let _ = write!(self.output, "\\u{:04x}", c as u32);
                }
                c => self.output.push(c),
            }
        }
        self.output.push(quote_char);
    }

    fn node_inline_width(&mut self, node: &Node<'_>, max_width: usize) -> Option<usize> {
        self.node_exact_inline_width(node).filter(|width| *width <= max_width)
    }

    fn node_exact_inline_width(&mut self, node: &Node<'_>) -> Option<usize> {
        let key = node as *const Node<'_> as usize;
        if let Some(width) = self.node_width_cache.get(&key) {
            return *width;
        }

        if !node.leading_comments.is_empty()
            || (!node.trailing_block_comments.is_empty() && self.flavor.prints_comments())
        {
            self.node_width_cache.insert(key, None);
            return None;
        }

        let width = match &node.kind {
            NodeKind::Scalar(scalar) => {
                let width = self.scalar_width(scalar);
                Some(width)
            }
            NodeKind::Object(object) => (!self.flavor.forces_expanded_layout())
                .then(|| self.object_exact_inline_width(object))
                .flatten(),
            NodeKind::Array(array) => (!self.flavor.forces_expanded_layout())
                .then(|| self.array_exact_inline_width(array))
                .flatten(),
        };
        self.node_width_cache.insert(key, width);
        width
    }

    fn scalar_width(&self, scalar: &Scalar<'_>) -> usize {
        match scalar {
            Scalar::String(value) => rendered_string_width(
                &value.value,
                match self.flavor {
                    JsonFlavor::Json | JsonFlavor::Jsonc | JsonFlavor::JsonStringify => {
                        QuoteStyle::Double
                    }
                    JsonFlavor::Json5 => self.options.quote_style,
                },
            ),
            Scalar::Number(raw) | Scalar::Identifier(raw) => raw.len(),
            Scalar::Boolean(true) => 4,
            Scalar::Boolean(false) => 5,
            Scalar::Null => 4,
        }
    }

    fn object_exact_inline_width(&mut self, object: &ObjectNode<'_>) -> Option<usize> {
        let key = object as *const ObjectNode<'_> as usize;
        if let Some(width) = self.object_width_cache.get(&key) {
            return *width;
        }

        if !object.trailing_comments.is_empty() {
            self.object_width_cache.insert(key, None);
            return None;
        }

        if object.members.is_empty() {
            self.object_width_cache.insert(key, Some(2));
            return Some(2);
        }

        let consistent = self.object_quotes_consistent(object);

        let mut width = 2;
        if self.options.bracket_spacing.value() {
            width += 2;
        }

        for (index, member) in object.members.iter().enumerate() {
            if !member.leading_comments.is_empty() {
                self.object_width_cache.insert(key, None);
                return None;
            }
            let value_width = match self.node_exact_inline_width(&member.value) {
                Some(width) => width,
                None => {
                    self.object_width_cache.insert(key, None);
                    return None;
                }
            };
            width += self.key_width(&member.key, consistent) + 2 + value_width;
            if index + 1 < object.members.len() {
                width += 2;
            }
        }

        self.object_width_cache.insert(key, Some(width));
        Some(width)
    }

    fn key_width(&self, key: &Key<'_>, consistent: bool) -> usize {
        match key {
            Key::Identifier(name) | Key::Number(name) => {
                if self.should_quote_key(key, consistent) {
                    rendered_string_width(name, self.key_quote_style())
                } else {
                    name.len()
                }
            }
            Key::String { value, preferred_quote, .. } => {
                if self.should_quote_key(key, consistent) {
                    rendered_string_width(
                        value,
                        self.key_quote_style_for_string(*preferred_quote, key),
                    )
                } else {
                    value.len()
                }
            }
        }
    }

    fn array_exact_inline_width(&mut self, array: &ArrayNode<'_>) -> Option<usize> {
        let key = array as *const ArrayNode<'_> as usize;
        if let Some(width) = self.array_width_cache.get(&key) {
            return *width;
        }

        if !array.trailing_comments.is_empty() {
            self.array_width_cache.insert(key, None);
            return None;
        }

        let mut width = 2;
        for (index, element) in array.elements.iter().enumerate() {
            let element_width = match self.node_exact_inline_width(element) {
                Some(width) => width,
                None => {
                    self.array_width_cache.insert(key, None);
                    return None;
                }
            };
            width += element_width;
            if index + 1 < array.elements.len() {
                width += 2;
            }
        }
        self.array_width_cache.insert(key, Some(width));
        Some(width)
    }

    fn object_quotes_consistent(&mut self, object: &ObjectNode<'_>) -> bool {
        if !self.options.quote_properties.is_consistent() {
            return false;
        }

        let key = object as *const ObjectNode<'_> as usize;
        if let Some(consistent) = self.object_consistent_cache.get(&key) {
            return *consistent;
        }

        let consistent =
            object.members.iter().any(|member| self.should_quote_key(&member.key, true));
        self.object_consistent_cache.insert(key, consistent);
        consistent
    }

    fn write_comments(&mut self, comments: &[Comment<'_>], indent: usize) {
        if !self.flavor.prints_comments() {
            return;
        }
        for comment in comments {
            self.write_indent(indent);
            if comment.has_newline {
                for (index, line) in comment.raw.lines().enumerate() {
                    if index > 0 {
                        self.push_line_ending();
                        self.write_indent(indent);
                    }
                    self.output.push_str(line);
                }
            } else {
                self.output.push_str(comment.raw);
            }
            self.push_line_ending();
        }
    }

    fn write_indent(&mut self, indent: usize) {
        match self.indent_unit {
            IndentUnit::Tab => {
                for _ in 0..indent {
                    self.output.push('\t');
                }
            }
            IndentUnit::Space(count) => {
                for _ in 0..indent * count {
                    self.output.push(' ');
                }
            }
        }
    }

    fn line_width_at_indent(&self, indent: usize) -> usize {
        let used = match self.indent_unit {
            IndentUnit::Tab => indent,
            IndentUnit::Space(count) => indent * count,
        };
        self.print_width.saturating_sub(used)
    }

    fn push_line_ending(&mut self) {
        self.output.push_str(self.line_ending);
    }

    fn ends_with_line_ending(&self) -> bool {
        self.output.ends_with(self.line_ending)
    }
}

fn rendered_string_width(value: &str, quote: QuoteStyle) -> usize {
    let mut width = 2;
    for ch in value.chars() {
        width += match ch {
            '\\' => 2,
            '"' if quote.is_double() => 2,
            '\'' if !quote.is_double() => 2,
            '\u{0008}' | '\u{000C}' | '\n' | '\r' | '\t' => 2,
            c if c < ' ' => 6,
            _ => ch.len_utf8(),
        };
    }
    width
}

fn string_needs_escaping(value: &str, quote: QuoteStyle) -> bool {
    value.chars().any(|ch| match ch {
        '\\' => true,
        '"' if quote.is_double() => true,
        '\'' if !quote.is_double() => true,
        '\u{0008}' | '\u{000C}' | '\n' | '\r' | '\t' => true,
        c if c < ' ' => true,
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use oxc_formatter::{FormatOptions, QuoteProperties, QuoteStyle, TrailingCommas};

    use super::{JsonFlavor, format_json};

    fn format_with_flavor(source: &str, flavor: JsonFlavor) -> String {
        format_json(source, "fixture".as_ref(), flavor, &FormatOptions::default()).unwrap()
    }

    #[test]
    fn formats_plain_json() {
        assert_eq!(format_with_flavor("{foo:'bar'}", JsonFlavor::Json), "{ \"foo\": \"bar\" }\n");
    }

    #[test]
    fn keeps_json_comments_but_removes_trailing_commas() {
        let source = r#"{
// first
"a": 1,
}"#;

        assert_eq!(format_with_flavor(source, JsonFlavor::Json), "{\n  // first\n  \"a\": 1\n}\n");
    }

    #[test]
    fn keeps_jsonc_comments_and_trailing_commas() {
        let source = r#"{
// first
"a": 1,
}"#;

        assert_eq!(
            format_with_flavor(source, JsonFlavor::Jsonc),
            "{\n  // first\n  \"a\": 1,\n}\n"
        );
    }

    #[test]
    fn rejects_comment_only_json_input() {
        let result = format_json(
            "// nope\n",
            "fixture".as_ref(),
            JsonFlavor::Json,
            &FormatOptions::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn supports_comment_only_jsonc_input() {
        assert_eq!(format_with_flavor("// ok\n", JsonFlavor::Jsonc), "// ok\n");
    }

    #[test]
    fn supports_json5_features() {
        let source = "{ unquoted: 'value', hex: +0xff, nan: NaN }";
        assert_eq!(
            format_with_flavor(source, JsonFlavor::Json5),
            "{ unquoted: \"value\", hex: +0xff, nan: NaN }\n"
        );
    }

    #[test]
    fn json5_quote_style_and_quote_props_follow_options() {
        let source = "{foo: 'bar', \"safe\": 1, \"baz-qux\": 2}";
        let options = FormatOptions {
            quote_style: QuoteStyle::Single,
            quote_properties: QuoteProperties::Preserve,
            ..FormatOptions::default()
        };
        assert_eq!(
            format_json(source, "fixture".as_ref(), JsonFlavor::Json5, &options).unwrap(),
            "{ foo: 'bar', 'safe': 1, 'baz-qux': 2 }\n"
        );
    }

    #[test]
    fn json5_consistent_quotes_all_keys() {
        let source = "{foo: 1, \"baz-qux\": 2}";
        let options = FormatOptions {
            quote_properties: QuoteProperties::Consistent,
            ..FormatOptions::default()
        };
        assert_eq!(
            format_json(source, "fixture".as_ref(), JsonFlavor::Json5, &options).unwrap(),
            "{ \"foo\": 1, \"baz-qux\": 2 }\n"
        );
    }

    #[test]
    fn json_stringify_uses_expanded_layout() {
        assert_eq!(
            format_with_flavor("{foo:'bar'}", JsonFlavor::JsonStringify),
            "{\n  \"foo\": \"bar\"\n}\n"
        );
    }

    #[test]
    fn json_stringify_rejects_comments() {
        let result = format_json(
            "{// nope\n\"a\":1}",
            "fixture".as_ref(),
            JsonFlavor::JsonStringify,
            &FormatOptions::default(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn detects_supported_json_file_kinds() {
        let cases = [
            ("data.json", Some(JsonFlavor::Json)),
            ("config.jsonc", Some(JsonFlavor::Jsonc)),
            ("settings.json5", Some(JsonFlavor::Json5)),
            (".swcrc", Some(JsonFlavor::Json)),
            ("workspace.code-workspace", Some(JsonFlavor::Jsonc)),
            ("schema.avsc", Some(JsonFlavor::Json)),
            ("config.importmap", Some(JsonFlavor::JsonStringify)),
            ("package.json", Some(JsonFlavor::JsonStringify)),
            ("composer.json", Some(JsonFlavor::JsonStringify)),
            ("custom.jsona", None),
        ];

        for (path, expected) in cases {
            assert_eq!(JsonFlavor::from_path(Path::new(path)), expected, "{path}");
        }
    }

    #[test]
    fn jsonc_trailing_comma_option_is_respected() {
        let source = "{foo: 1, bar: 2,}";
        let options = FormatOptions { trailing_commas: TrailingCommas::None, ..Default::default() };
        assert_eq!(
            format_json(source, "fixture".as_ref(), JsonFlavor::Jsonc, &options).unwrap(),
            "{ \"foo\": 1, \"bar\": 2 }\n"
        );
    }
}

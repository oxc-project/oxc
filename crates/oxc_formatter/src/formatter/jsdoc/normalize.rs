use std::borrow::Cow;
use std::fmt::Write as _;

/// Normalize JSDoc tag aliases to their canonical form.
/// Matches prettier-plugin-jsdoc's `TAGS_SYNONYMS` from `roles.ts`.
pub fn normalize_tag_kind(kind: &str) -> &str {
    match kind {
        "return" => "returns",
        "arg" | "argument" | "params" => "param",
        "yield" => "yields",
        "prop" => "property",
        "constructor" => "class",
        "const" => "constant",
        "desc" => "description",
        "host" => "external",
        "fileoverview" | "overview" => "file",
        "emits" => "fires",
        "func" | "method" => "function",
        "var" => "member",
        "virtual" => "abstract",
        "exception" => "throws",
        "examples" => "example",
        "hidden" => "ignore",
        "memberOf" => "memberof",
        // Note: @augments and @extends are NOT synonyms in the plugin.
        // They have different sort weights (20 vs 33).
        // @linkcode, @linkplain are also NOT normalized to @link.
        _ => kind,
    }
}

/// Normalize markdown emphasis markers:
/// - `__text__` → `**text**` (double underscore bold → asterisk bold)
/// - `*text*` → `_text_` (single asterisk italic → underscore italic)
///
/// Matches prettier-plugin-jsdoc which normalizes emphasis through remark.
/// Bold uses `**`, italic uses `_`.
pub fn normalize_markdown_emphasis(text: &str) -> Cow<'_, str> {
    if !text.contains("__") && !text.contains('*') {
        return Cow::Borrowed(text);
    }

    // Work with bytes directly — all significant chars (_, *, `, whitespace)
    // are ASCII single-byte, so byte-level mutation is safe and uses ~3x less
    // memory than Vec<char>.
    let mut bytes: Vec<u8> = text.as_bytes().to_vec();
    let len = bytes.len();
    let mut i = 0;
    let mut in_code = false;
    let mut changed = false;

    // First pass: convert `__` → `**`
    while i < len {
        if bytes[i] == b'`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if in_code {
            i += 1;
            continue;
        }
        if bytes[i] == b'_' && i + 1 < len && bytes[i + 1] == b'_' {
            bytes[i] = b'*';
            bytes[i + 1] = b'*';
            changed = true;
            i += 2;
            continue;
        }
        i += 1;
    }

    // Second pass: convert single `*text*` → `_text_`
    // Skip `**` (bold) and content inside backticks.
    in_code = false;
    i = 0;
    while i < len {
        if bytes[i] == b'`' {
            in_code = !in_code;
            i += 1;
            continue;
        }
        if in_code {
            i += 1;
            continue;
        }

        // Skip `**` (bold markers)
        if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            continue;
        }

        // Single `*` — check if it's an opening emphasis marker:
        // Must be followed by a non-whitespace character
        if bytes[i] == b'*' && i + 1 < len && !bytes[i + 1].is_ascii_whitespace() {
            // Look for matching closing `*`
            let opener = i;
            let mut j = opener + 1;
            while j < len {
                if bytes[j] == b'`' {
                    // Skip inline code spans
                    j += 1;
                    while j < len && bytes[j] != b'`' {
                        j += 1;
                    }
                    if j < len {
                        j += 1;
                    }
                    continue;
                }
                // Skip `**` inside emphasis
                if bytes[j] == b'*' && j + 1 < len && bytes[j + 1] == b'*' {
                    j += 2;
                    continue;
                }
                // Found closing `*`: must be preceded by non-whitespace
                if bytes[j] == b'*' && j > opener + 1 && !bytes[j - 1].is_ascii_whitespace() {
                    bytes[opener] = b'_';
                    bytes[j] = b'_';
                    changed = true;
                    i = j + 1;
                    break;
                }
                j += 1;
            }
            if i <= opener {
                i = opener + 1;
            }
            continue;
        }

        i += 1;
    }

    // If no bytes were actually modified, return the original text without allocation.
    if !changed {
        return Cow::Borrowed(text);
    }
    // We only replaced ASCII bytes (_, *) with other ASCII bytes (*, _),
    // so UTF-8 validity is preserved.
    Cow::Owned(String::from_utf8(bytes).unwrap())
}

/// Capitalize the first ASCII lowercase letter of a string.
/// Skips if the string starts with a backtick (inline code) or a URL.
/// Handles `"- "` prefix iteratively: `"- - hello"` → `"- - Hello"` with a single allocation.
pub fn capitalize_first(s: &str) -> Cow<'_, str> {
    if s.is_empty() || s.starts_with('`') || s.starts_with("http://") || s.starts_with("https://") {
        return Cow::Borrowed(s);
    }

    // Handle dash-prefix iteratively: count leading "- " prefixes,
    // capitalize the inner text once, build result with single allocation.
    let mut prefix_len = 0;
    let mut remaining = s;
    while let Some(rest) = remaining.strip_prefix("- ") {
        prefix_len += 2;
        remaining = rest;
    }

    if prefix_len > 0 {
        // `remaining` no longer starts with "- ", so this won't recurse into dash handling.
        let capitalized = capitalize_first(remaining);
        if matches!(capitalized, Cow::Borrowed(_)) {
            // Inner text was unchanged — return original string as-is
            return Cow::Borrowed(s);
        }
        let mut result = String::with_capacity(prefix_len + capitalized.len());
        result.push_str(&s[..prefix_len]);
        result.push_str(&capitalized);
        return Cow::Owned(result);
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {
            let mut result = String::with_capacity(s.len());
            result.push(c.to_ascii_uppercase());
            result.push_str(chars.as_str());
            Cow::Owned(result)
        }
        _ => Cow::Borrowed(s),
    }
}

/// Normalize JSDoc type expression syntax:
/// - `?Type` → `Type | null`
/// - `Type?` → `Type | null` (except inside quotes)
/// - `*` → `any`
/// - `...Type` → `...Type` (with normalized spacing)
/// - `Array.<T>` / `Array<T>` → `T[]`
/// - `Foo.<T>` → `Foo<T>` (Closure Compiler dot syntax)
/// - Single quotes → double quotes in import() paths
/// - Normalize whitespace and operator spacing
///
/// Matches prettier-plugin-jsdoc's `convertToModernType()` which uses
/// `withoutStrings()` to protect quoted strings during transformation.
pub fn normalize_type(type_str: &str) -> Cow<'_, str> {
    normalize_type_impl(type_str, true)
}

/// Normalize type but preserve original quotes.
/// Used for `@type`, `@typedef`, `@satisfies` where the plugin keeps the type
/// mostly as-is (via `getUpdatedType()` in stringify.ts).
pub fn normalize_type_preserve_quotes(type_str: &str) -> Cow<'_, str> {
    normalize_type_impl(type_str, false)
}

fn normalize_type_impl(type_str: &str, convert_quotes: bool) -> Cow<'_, str> {
    // Fast path: common simple types need no transformation at all.
    let trimmed = type_str.trim();
    if matches!(
        trimmed,
        "string"
            | "number"
            | "boolean"
            | "void"
            | "undefined"
            | "null"
            | "object"
            | "never"
            | "unknown"
            | "any"
            | "bigint"
            | "symbol"
            | "this"
    ) {
        return Cow::Borrowed(trimmed);
    }

    // Extended fast path: simple identifier types need no transformation.
    // Covers dotted names (`Estree.Node`), plain identifiers (`Object`, `Node`),
    // and simple generic types (`Map<string, number>`) — none of which contain
    // characters that trigger any normalization phase.
    if is_already_normalized(trimmed) {
        return Cow::Borrowed(trimmed);
    }

    // Phase 1: Protect quoted strings, run core transforms, restore strings.
    // This matches the plugin's convertToModernType() inside withoutStrings().
    let transformed = without_strings(type_str, normalize_type_inner);
    // Phase 2: Convert import() path quotes (simulating Prettier's TS parser).
    let quoted =
        if convert_quotes { normalize_type_quotes(&transformed) } else { Cow::Owned(transformed) };
    // Phase 3: Unquote object property names that are valid JS identifiers.
    // The plugin's formatType() uses Prettier's TS parser which strips unnecessary quotes.
    let unquoted = if convert_quotes { unquote_object_property_names(&quoted) } else { quoted };
    // Phase 4: Format inline object type spacing (simulating Prettier's TS parser).
    // { key:value } → { key: value }
    Cow::Owned(format_inline_object_type(&unquoted).into_owned())
}

/// Protect quoted strings during type transformation.
/// Matches prettier-plugin-jsdoc's `withoutStrings()` from `utils.ts`:
/// 1. Replace all quoted strings (`"..."` and `'...'`) with `String$N$` placeholders
/// 2. If backticks found after replacement, bail and return original
/// 3. Run transformation on the placeholder-filled string
/// 4. Restore original quoted strings
fn without_strings(type_str: &str, transform: impl FnOnce(&str) -> String) -> String {
    // Fast path: no quotes at all — skip the entire placeholder machinery.
    if !type_str.contains('\'') && !type_str.contains('"') {
        // Bail on backticks (template literal types)
        if type_str.contains('`') {
            return type_str.to_string();
        }
        return transform(type_str);
    }

    let mut strings: Vec<&str> = Vec::new();
    let mut modified = String::with_capacity(type_str.len());
    let bytes = type_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];
        if ch == b'"' || ch == b'\'' {
            let quote = ch;
            let start = i;
            i += 1;
            // Scan to closing quote, handling backslash escapes
            while i < len {
                if bytes[i] == b'\\' {
                    i += 1; // skip backslash
                    if i < len {
                        i += 1; // skip escaped char only if present
                    }
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            let matched = &type_str[start..i.min(len)];
            strings.push(matched);
            write!(modified, "String${}$", strings.len() - 1).unwrap();
        } else {
            let ch = type_str[i..].chars().next().unwrap();
            modified.push(ch);
            i += ch.len_utf8();
        }
    }

    // Bail on backticks (template literal types) — can't handle correctly
    if modified.contains('`') {
        return type_str.to_string();
    }

    let result = transform(&modified);

    // Restore original quoted strings in a single pass
    replace_placeholders(&result, "String$", &strings)
}

/// Single-pass placeholder replacement.
/// Scans `input` for occurrences of `{prefix}{N}{suffix_char}` (e.g. `String$0$`, `String$12$`)
/// and replaces each with `originals[N]`. Characters outside placeholders are copied verbatim.
fn replace_placeholders(input: &str, prefix: &str, originals: &[&str]) -> String {
    if originals.is_empty() {
        return input.to_string();
    }

    let prefix_bytes = prefix.as_bytes();
    let prefix_len = prefix_bytes.len();
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Check if we're at a placeholder start
        if i + prefix_len < len && &bytes[i..i + prefix_len] == prefix_bytes {
            // Try to parse digits after the prefix
            let digit_start = i + prefix_len;
            let mut digit_end = digit_start;
            while digit_end < len && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            // Must have at least one digit and a trailing '$'
            if digit_end > digit_start
                && digit_end < len
                && bytes[digit_end] == b'$'
                && let Ok(idx) = input[digit_start..digit_end].parse::<usize>()
                && idx < originals.len()
            {
                result.push_str(originals[idx]);
                i = digit_end + 1; // skip past trailing '$'
                continue;
            }
        }
        // Not a placeholder — copy the character
        let ch = input[i..].chars().next().unwrap();
        result.push(ch);
        i += ch.len_utf8();
    }

    result
}

/// Core type normalization (without quote conversion).
fn normalize_type_inner(type_str: &str) -> String {
    let trimmed = type_str.trim();

    // `*` alone means `any`
    if trimmed == "*" {
        return "any".to_string();
    }

    // String literal types (e.g. `"some text"`, `'foo'`) — return as-is.
    // The outer without_strings() already protects these with placeholders,
    // but if we reach here with a string literal, preserve it unchanged.
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return trimmed.to_string();
    }

    // Rest/spread prefix: `... type` → `...normalizedType`
    if let Some(stripped) = trimmed.strip_prefix("...") {
        let rest = stripped.trim_start();
        if !rest.is_empty() {
            let normalized = normalize_type_inner(rest);
            // Wrap in parens if contains top-level union
            if needs_parens_for_union(&normalized) {
                return format!("...({normalized})");
            }
            return format!("...{normalized}");
        }
        return trimmed.to_string();
    }

    // `?Type` (nullable prefix) → `Type | null`
    if let Some(rest) = trimmed.strip_prefix('?') {
        let inner = rest.trim();
        if !inner.is_empty() {
            return format!("{} | null", normalize_type_core(inner));
        }
    }

    // `Type?` (nullable suffix, Closure Compiler style) → `Type | null`
    // But not inside quotes (e.g. `import("foo?")`)
    if trimmed.ends_with('?') && !contains_quotes(trimmed) {
        let inner = &trimmed[..trimmed.len() - 1];
        if !inner.is_empty() && !inner.contains('?') {
            return format!("{} | null", normalize_type_core(inner));
        }
    }

    normalize_type_core(trimmed)
}

/// Apply core normalizations: Array conversion, dot removal, then whitespace normalization.
/// For union types, normalize each member independently.
fn normalize_type_core(type_str: &str) -> String {
    let trimmed = type_str.trim();

    if trimmed == "*" {
        return "any".to_string();
    }

    // Check for top-level union: split at `|`, normalize each part, then join
    let parts = split_at_top_level_pipe(trimmed);
    if parts.len() > 1 {
        let normalized_parts: Vec<String> =
            parts.iter().map(|p| normalize_type_core(p.trim())).collect();
        return normalized_parts.join(" | ");
    }

    // Remove Closure Compiler `.` before `<` in generic types: `Foo.<T>` → `Foo<T>`
    let cleaned = remove_closure_dot_generics(trimmed);

    // Global Array conversion: find and replace `Array<...>` patterns anywhere in the string.
    // Matches prettier-plugin-jsdoc's `convertToModernType()` which uses a regex in a
    // `while(changed)` loop to iteratively convert innermost Array<...> patterns.
    let converted = convert_array_types_globally(&cleaned);

    normalize_type_whitespace(&converted).into_owned()
}

/// Convert `Array<T>` patterns anywhere in a type string to `T[]`.
/// Runs iteratively (like the plugin's `while(changed)` loop) to handle nested arrays.
fn convert_array_types_globally(type_str: &str) -> Cow<'_, str> {
    let Some(first) = replace_one_array_pattern(type_str) else {
        return Cow::Borrowed(type_str);
    };
    let mut result = first;
    while let Some(new_result) = replace_one_array_pattern(&result) {
        result = new_result;
    }
    Cow::Owned(result)
}

/// Find and replace one `Array<...>` pattern in the string.
/// Returns `None` if no pattern found.
/// The pattern must be preceded by a non-word char (or start of string).
fn replace_one_array_pattern(type_str: &str) -> Option<String> {
    // Find all positions of "Array" that could be `Array<...>`
    let mut search_start = 0;
    while let Some(pos) = type_str[search_start..].find("Array") {
        let pos = search_start + pos;

        // Check that Array is preceded by a non-word char (matching plugin's regex prefix)
        if pos > 0 {
            let prev = type_str.as_bytes()[pos - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' || prev == b'$' || prev > 0x7F {
                search_start = pos + 5;
                continue;
            }
        }

        // Check for `Array<` or `Array.<`
        let after_array = &type_str[pos + 5..];
        let (inner_start, _dot_len) = if after_array.starts_with(".<") {
            (pos + 7, 2) // `Array.<`
        } else if after_array.starts_with('<') {
            (pos + 6, 1) // `Array<`
        } else {
            search_start = pos + 5;
            continue;
        };

        // Find matching closing `>`
        let inner_str = &type_str[inner_start..];
        if let Some(end_offset) = find_matching_close_angle(inner_str) {
            let inner = &type_str[inner_start..inner_start + end_offset];
            let after = &type_str[inner_start + end_offset + 1..];
            let prefix = &type_str[..pos];

            // Normalize the inner type recursively
            let normalized_inner = normalize_type_inner(inner);

            // Wrap in parens if needed
            let array_elem = if needs_parens_for_array(&normalized_inner) {
                format!("({normalized_inner})[]")
            } else {
                format!("{normalized_inner}[]")
            };

            return Some(format!("{prefix}{array_elem}{after}"));
        }

        search_start = pos + 5;
    }
    None
}

/// Find the position of the matching closing `>` for an angle bracket expression.
/// Returns the offset of `>` within the string (not including the opening `<`).
fn find_matching_close_angle(s: &str) -> Option<usize> {
    let mut depth: i32 = 1;
    let bytes = s.as_bytes();
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                // Skip `=>` (arrow function syntax)
                if i > 0 && bytes[i - 1] == b'=' {
                    continue;
                }
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Split a type string at top-level `|` operators (not inside `<>`, `()`, `{}`, `[]`).
fn split_at_top_level_pipe(type_str: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let bytes = type_str.as_bytes();

    for (i, ch) in type_str.char_indices() {
        match ch {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' => {
                // Don't count `=>` as closing bracket
                if i > 0 && bytes[i - 1] == b'=' {
                    continue;
                }
                depth -= 1;
            }
            '|' if depth == 0 => {
                parts.push(&type_str[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&type_str[start..]);
    parts
}

/// Normalize a type for return-like tags (returns, yields, throws).
/// Handles `type=` → `type | undefined` (Closure optional return syntax).
pub fn normalize_type_return(type_str: &str) -> Cow<'_, str> {
    let trimmed = type_str.trim();
    if trimmed.ends_with('=') && !trimmed.ends_with("=>") && !contains_quotes(trimmed) {
        let inner = &trimmed[..trimmed.len() - 1];
        if !inner.is_empty() {
            let normalized = normalize_type(inner);
            return Cow::Owned(format!("{normalized} | undefined"));
        }
    }
    normalize_type(trimmed)
}

/// Check for optional suffix `=` on a param type.
/// Returns `(type_without_equals, is_optional)`.
pub fn strip_optional_type_suffix(type_str: &str) -> (&str, bool) {
    let trimmed = type_str.trim();
    if trimmed.ends_with('=') && !trimmed.ends_with("=>") && !contains_quotes(trimmed) {
        let inner = trimmed[..trimmed.len() - 1].trim_end();
        if !inner.is_empty() {
            return (inner, true);
        }
    }
    (trimmed, false)
}

/// Check if a type string needs parenthesization when used as an array element type.
/// This includes top-level unions (`|`) and function types (`=>`).
fn needs_parens_for_array(type_str: &str) -> bool {
    if type_str.starts_with('(') && type_str.ends_with(')') {
        return false;
    }
    needs_parens_for_union(type_str) || contains_top_level_arrow(type_str)
}

/// Check if a type string contains a top-level `=>` (function type).
fn contains_top_level_arrow(type_str: &str) -> bool {
    if !type_str.contains("=>") {
        return false;
    }
    let bytes = type_str.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'<' | b'[' | b'{' => depth += 1,
            b')' | b'>' | b']' | b'}' => depth -= 1,
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'>' => return true,
            _ => {}
        }
    }
    false
}

/// Check if a type string contains a top-level `|` (union) that needs parenthesization.
fn needs_parens_for_union(type_str: &str) -> bool {
    if !type_str.contains('|') {
        return false;
    }
    if type_str.starts_with('(') && type_str.ends_with(')') {
        return false;
    }

    let mut depth = 0i32;
    let mut prev = '\0';
    for ch in type_str.chars() {
        match ch {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' if prev != '=' => depth -= 1,
            '|' if depth == 0 => return true,
            _ => {}
        }
        prev = ch;
    }
    false
}

/// Remove Closure Compiler `.` before `<` in generic type syntax.
/// e.g., `Object.<String, Number>` → `Object<String, Number>`
fn remove_closure_dot_generics(type_str: &str) -> Cow<'_, str> {
    if !type_str.contains(".<") {
        return Cow::Borrowed(type_str);
    }
    // Don't modify content inside quotes
    let mut result = String::with_capacity(type_str.len());
    let mut in_quote = false;
    let mut quote_byte = b'"';
    let bytes = type_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if in_quote {
            if b == quote_byte {
                in_quote = false;
            }
            let ch = type_str[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        } else if b == b'"' || b == b'\'' {
            in_quote = true;
            quote_byte = b;
            result.push(b as char);
            i += 1;
        } else if b == b'.' && i + 1 < len && bytes[i + 1] == b'<' {
            // Skip the `.` before `<`
            i += 1;
        } else if b.is_ascii() {
            result.push(b as char);
            i += 1;
        } else {
            let ch = type_str[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    Cow::Owned(result)
}

fn contains_quotes(s: &str) -> bool {
    s.contains('"') || s.contains('\'')
}

/// Convert single quotes to double quotes in JSDoc type expressions.
/// Matches Prettier's TS parser behavior which normalizes quote style.
/// Converts:
/// - `import('foo')` → `import("foo")`
/// - `'string literal'` → `"string literal"` (standalone string literal types)
///
/// Does NOT convert single quotes inside double-quoted strings (already protected by
/// `without_strings()` in the transformation phase).
fn normalize_type_quotes(type_str: &str) -> Cow<'_, str> {
    if !type_str.contains('\'') {
        return Cow::Borrowed(type_str);
    }

    let mut result = String::with_capacity(type_str.len());
    let bytes = type_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\'' {
            // Convert single-quoted string to double-quoted
            result.push('"');
            i += 1;
            while i < len && bytes[i] != b'\'' {
                if bytes[i] == b'\\' && i + 1 < len {
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                } else {
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                }
            }
            if i < len && bytes[i] == b'\'' {
                result.push('"');
                i += 1;
            }
        } else if bytes[i] == b'"' {
            // Skip over double-quoted strings entirely (preserve as-is)
            result.push('"');
            i += 1;
            while i < len && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < len {
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                } else {
                    let ch = type_str[i..].chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                }
            }
            if i < len && bytes[i] == b'"' {
                result.push('"');
                i += 1;
            }
        } else {
            let ch = type_str[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }
    Cow::Owned(result)
}

/// Check if a string is a valid JavaScript identifier.
fn is_valid_js_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Remove unnecessary quotes from object property names in type expressions.
/// `"userId": string` → `userId: string` when the property name is a valid identifier.
/// This simulates Prettier's TS parser behavior.
fn unquote_object_property_names(type_str: &str) -> Cow<'_, str> {
    if !type_str.contains('"') {
        return Cow::Borrowed(type_str);
    }

    let bytes = type_str.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'"' {
            // Found a double-quoted string — check if it's a property name
            let start = i + 1;
            i += 1;
            // Find closing quote
            let mut end = None;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2; // skip escape
                    continue;
                }
                if bytes[i] == b'"' {
                    end = Some(i);
                    i += 1;
                    break;
                }
                i += 1;
            }

            if let Some(end_pos) = end {
                let quoted_content = &type_str[start..end_pos];

                // Check if followed by `:` (with optional whitespace) — this is a property name
                let after = &type_str[i..];
                let after_trimmed = after.trim_start();
                if after_trimmed.starts_with(':') && is_valid_js_identifier(quoted_content) {
                    // Unquote: write property name without quotes
                    result.push_str(quoted_content);
                } else {
                    // Not a property name or not a valid identifier — keep quotes
                    result.push('"');
                    result.push_str(quoted_content);
                    result.push('"');
                }
            } else {
                // Unterminated string — push as-is
                result.push('"');
                result.push_str(&type_str[start..]);
            }
        } else {
            let ch = type_str[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    Cow::Owned(result)
}

/// Format inline object types with proper spacing.
/// `{foo:string}` → `{ foo: string }`
/// Only handles single-level object types that don't contain newlines.
fn format_inline_object_type(type_str: &str) -> Cow<'_, str> {
    let trimmed = type_str.trim();
    // Only apply to `{...}` patterns (not nested `{{...}}` which is handled by wrap_object_type)
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') || trimmed.contains('\n') {
        return Cow::Borrowed(type_str);
    }

    // Skip `{{ }}` double-brace types — these are handled by serialize.rs wrap_object_type
    if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
        // Format the inner content of {{ }}
        let inner = &trimmed[1..trimmed.len() - 1]; // Strip one level of braces
        let formatted_inner = format_object_body(inner);
        return Cow::Owned(format!("{{{formatted_inner}}}"));
    }

    // Single brace: format { key: value; ... }
    format_object_body(trimmed)
}

/// Format an object type body `{ key: value; key2: value2 }` with proper spacing.
fn format_object_body(obj_str: &str) -> Cow<'_, str> {
    let trimmed = obj_str.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Cow::Borrowed(obj_str);
    }

    let inner = trimmed[1..trimmed.len() - 1].trim();
    if inner.is_empty() {
        return Cow::Borrowed("{}");
    }

    // Split at top-level `;` or `,`
    let mut fields = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    let bytes = inner.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'<' | b'[' | b'{' => depth += 1,
            b')' | b'>' | b']' | b'}' => depth -= 1,
            b';' | b',' if depth == 0 => {
                let field = inner[start..i].trim();
                if !field.is_empty() {
                    fields.push(field);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let last = inner[start..].trim();
    if !last.is_empty() {
        fields.push(last);
    }

    if fields.is_empty() {
        return Cow::Owned(format!("{{ {inner} }}"));
    }

    // Format each field: ensure space after `:` in key-value pairs
    let formatted_fields: Vec<Cow<'_, str>> = fields
        .iter()
        .map(|field| {
            // Find the `:` separating key from value (skip `:` inside nested types)
            let mut fd = 0i32;
            let fb = field.as_bytes();
            for (i, &b) in fb.iter().enumerate() {
                match b {
                    b'(' | b'<' | b'[' | b'{' => fd += 1,
                    b')' | b'>' | b']' | b'}' => fd -= 1,
                    b':' if fd == 0 => {
                        let key = field[..i].trim();
                        let value = field[i + 1..].trim();
                        return Cow::Owned(format!("{key}: {value}"));
                    }
                    _ => {}
                }
            }
            Cow::Borrowed(*field)
        })
        .collect();

    // Use `;` delimiter for consistency
    let mut body = String::new();
    for (i, f) in formatted_fields.iter().enumerate() {
        if i > 0 {
            body.push_str("; ");
        }
        body.push_str(f);
    }
    Cow::Owned(format!("{{ {body} }}"))
}

/// Fast check: is the type expression already whitespace-normalized?
/// Returns `true` when the string has no leading/trailing whitespace, no consecutive spaces,
/// no non-space ASCII whitespace (tabs, newlines), no non-ASCII whitespace, and `|`, `&`, `=>`
/// already have spaces around them.
fn is_whitespace_normalized(s: &str) -> bool {
    let bytes = s.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return true;
    }
    // Leading/trailing whitespace
    if bytes[0].is_ascii_whitespace() || bytes[len - 1].is_ascii_whitespace() {
        return false;
    }
    let mut i = 0;
    while i < len {
        let b = bytes[i];
        if b.is_ascii_whitespace() {
            if b != b' ' {
                return false; // tabs, newlines
            }
            if i + 1 < len && bytes[i + 1] == b' ' {
                return false; // consecutive spaces
            }
        } else if b == b'|' || b == b'&' {
            if i == 0 || bytes[i - 1] != b' ' {
                return false;
            }
            if i + 1 >= len || bytes[i + 1] != b' ' {
                return false;
            }
        } else if b == b'=' && i + 1 < len && bytes[i + 1] == b'>' {
            if i == 0 || bytes[i - 1] != b' ' {
                return false;
            }
            if i + 2 >= len || bytes[i + 2] != b' ' {
                return false;
            }
            i += 2; // skip `>`
            continue;
        } else if !b.is_ascii() {
            let ch = s[i..].chars().next().unwrap();
            if ch.is_whitespace() {
                return false; // non-ASCII whitespace
            }
            i += ch.len_utf8();
            continue;
        }
        i += 1;
    }
    true
}

/// Normalize whitespace within a type expression:
/// - Collapse runs of whitespace to a single space
/// - Add spaces around `|` and `&` operators if missing
/// - Trim leading/trailing whitespace
pub fn normalize_type_whitespace(type_str: &str) -> Cow<'_, str> {
    let trimmed = type_str.trim();
    if is_whitespace_normalized(trimmed) {
        return Cow::Borrowed(trimmed);
    }

    // Single pass: collapse whitespace AND ensure spaces around `|`, `&`, `=>`
    // while preserving `// comments` verbatim through newline.
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len + 8);
    let mut prev_was_space = false;
    let mut i = 0;

    while i < len {
        let b = bytes[i];

        // Detect `// comment` — preserve verbatim through newline
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            while i < len && bytes[i] != b'\n' {
                let ch = trimmed[i..].chars().next().unwrap();
                result.push(ch);
                i += ch.len_utf8();
            }
            if i < len && bytes[i] == b'\n' {
                result.push('\n');
                i += 1;
            }
            prev_was_space = false;
            continue;
        }

        // Handle `=>` arrow — ensure spaces around it
        if b == b'=' && i + 1 < len && bytes[i + 1] == b'>' {
            // Ensure space before: if last char emitted wasn't a space, add one
            if !prev_was_space && !result.is_empty() {
                result.push(' ');
            }
            result.push_str("=>");
            prev_was_space = false;
            i += 2;
            // Ensure space after: peek at next non-whitespace-producing position
            // We'll emit a space now; if the next char is whitespace, the
            // collapsing logic will suppress the duplicate.
            if i < len && !bytes[i].is_ascii_whitespace() {
                result.push(' ');
                prev_was_space = true;
            }
            continue;
        }

        // Handle `|` and `&` — ensure spaces around them
        if b == b'|' || b == b'&' {
            if !prev_was_space && !result.is_empty() {
                result.push(' ');
            }
            result.push(b as char);
            prev_was_space = false;
            i += 1;
            if i < len && !bytes[i].is_ascii_whitespace() {
                result.push(' ');
                prev_was_space = true;
            }
            continue;
        }

        // ASCII whitespace — collapse runs to a single space
        if b.is_ascii_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
            i += 1;
            continue;
        }

        // Plain ASCII character
        if b.is_ascii() {
            result.push(b as char);
            prev_was_space = false;
            i += 1;
            continue;
        }

        // Non-ASCII: could be whitespace (e.g. NBSP) or regular char
        let ch = trimmed[i..].chars().next().unwrap();
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
        i += ch.len_utf8();
    }

    Cow::Owned(result)
}

/// Check if a type string is already normalized and needs no transformation.
///
/// Returns `true` for types composed entirely of identifier characters (alphanumeric,
/// `_`, `$`, `.`), angle brackets (`<`, `>`), square brackets (`[`, `]`), commas, and
/// single spaces — as long as none of the transformation triggers are present.
///
/// This covers common JSDoc types like `Object`, `Node`, `Estree.Node`, `CommentContext`,
/// `Map<string, number>`, `string[]`, `Promise<void>`, etc.
fn is_already_normalized(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Quick rejection: any character that triggers a transformation phase.
    // `?` → nullable, `'`/`"` → quotes, `{`/`}` → object types,
    // `*` → any, `=` → optional/arrow, `&` → intersection spacing,
    // `|` → union spacing, `!` → non-null assertion
    let bytes = s.as_bytes();
    for &b in bytes {
        match b {
            b'?' | b'\'' | b'"' | b'{' | b'}' | b'*' | b'=' | b'&' | b'|' | b'!' | b'\n' => {
                return false;
            }
            _ => {}
        }
    }

    // Reject `Array<` and `Array.<` which trigger array conversion
    if s.contains("Array<") || s.contains("Array.<") {
        return false;
    }

    // Reject `...` prefix (rest/spread)
    if s.starts_with("...") {
        return false;
    }

    // Reject consecutive spaces (would be collapsed by whitespace normalizer)
    if s.contains("  ") {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tag_kind() {
        assert_eq!(normalize_tag_kind("return"), "returns");
        assert_eq!(normalize_tag_kind("arg"), "param");
        assert_eq!(normalize_tag_kind("argument"), "param");
        assert_eq!(normalize_tag_kind("yield"), "yields");
        assert_eq!(normalize_tag_kind("prop"), "property");
        assert_eq!(normalize_tag_kind("param"), "param");
        assert_eq!(normalize_tag_kind("returns"), "returns");
        assert_eq!(normalize_tag_kind("custom"), "custom");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("Hello"), "Hello");
        assert_eq!(capitalize_first("`code`"), "`code`");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("123"), "123");
        assert_eq!(capitalize_first("a"), "A");
        // Dash prefix handling (matches upstream's capitalizer)
        assert_eq!(capitalize_first("- hello"), "- Hello");
        assert_eq!(capitalize_first("- Hello"), "- Hello");
        assert_eq!(capitalize_first("- `code`"), "- `code`");
    }

    #[test]
    fn test_normalize_tag_kind_memberof() {
        assert_eq!(normalize_tag_kind("memberOf"), "memberof");
    }

    #[test]
    fn test_normalize_type() {
        assert_eq!(normalize_type("*"), "any");
        assert_eq!(normalize_type("?string"), "string | null");
        assert_eq!(normalize_type("?Number"), "Number | null");
        assert_eq!(normalize_type("string?"), "string | null");
        assert_eq!(normalize_type("?undefined"), "undefined | null");
        assert_eq!(normalize_type("string | number"), "string | number");
    }

    #[test]
    fn test_normalize_type_quotes() {
        assert_eq!(normalize_type("import('axios')"), "import(\"axios\")");
        assert_eq!(normalize_type("import('../types').Foo"), "import(\"../types\").Foo");
        assert_eq!(normalize_type("import(\"axios\")"), "import(\"axios\")");
    }

    #[test]
    fn test_normalize_array_type() {
        assert_eq!(normalize_type("Array.<String>"), "String[]");
        assert_eq!(normalize_type("Array<String>"), "String[]");
        assert_eq!(normalize_type("Array<(String | Number)>"), "(String | Number)[]");
        assert_eq!(normalize_type("Array<String|Number>"), "(String | Number)[]");
        // Arrow function inside Array<> should not confuse the > matching
        assert_eq!(
            normalize_type("Array<(element: HTMLElement) => boolean>"),
            "((element: HTMLElement) => boolean)[]"
        );
        // Array conversion inside union members
        assert_eq!(normalize_type("Array<number> | Array<string>"), "number[] | string[]");
        assert_eq!(
            normalize_type(
                "Array<(item: Foo<Bar>) => Bar<number>> | Array<number> | Array<string>"
            ),
            "((item: Foo<Bar>) => Bar<number>)[] | number[] | string[]"
        );
    }

    #[test]
    fn test_normalize_rest_type() {
        assert_eq!(normalize_type("... *"), "...any");
        assert_eq!(normalize_type("... number"), "...number");
        assert_eq!(normalize_type("... (string|number)"), "...(string | number)");
        assert_eq!(normalize_type("... string|number"), "...(string | number)");
    }

    #[test]
    fn test_normalize_type_return() {
        assert_eq!(normalize_type_return("string="), "string | undefined");
        assert_eq!(normalize_type_return("string"), "string");
        assert_eq!(normalize_type_return("number="), "number | undefined");
    }

    #[test]
    fn test_strip_optional_type_suffix() {
        assert_eq!(strip_optional_type_suffix("number="), ("number", true));
        assert_eq!(strip_optional_type_suffix("string"), ("string", false));
        assert_eq!(strip_optional_type_suffix("()=>void"), ("()=>void", false));
    }

    #[test]
    fn test_normalize_type_whitespace() {
        assert_eq!(normalize_type_whitespace("string"), "string");
        assert_eq!(normalize_type_whitespace("  string  |  number  "), "string | number");
        assert_eq!(normalize_type_whitespace("string|number"), "string | number");
        assert_eq!(normalize_type_whitespace("Array< string >"), "Array< string >");
        assert_eq!(normalize_type_whitespace("  a   b  "), "a b");
        assert_eq!(normalize_type_whitespace("A&B"), "A & B");
    }
}

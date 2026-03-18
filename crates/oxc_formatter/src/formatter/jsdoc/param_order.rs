use oxc_ast::Comment;

/// Reorder @param tags to match the function signature parameter order.
/// Only reorders when:
/// - All @param tags have type annotations (the plugin skips typeless params)
/// - The @param names exactly match the function parameters (same set, different order)
pub(super) fn reorder_param_tags(
    effective_tags: &mut [(&oxc_jsdoc::parser::JSDocTag<'_>, &str)],
    comment: &Comment,
    source_text: &str,
) {
    // Find consecutive @param tags
    let param_start = effective_tags.iter().position(|(_, kind)| *kind == "param");
    let Some(param_start) = param_start else {
        return;
    };
    let param_end = effective_tags[param_start..]
        .iter()
        .position(|(_, kind)| *kind != "param")
        .map_or(effective_tags.len(), |pos| param_start + pos);

    if param_end - param_start < 2 {
        return;
    }

    let param_tags = &effective_tags[param_start..param_end];

    // Parse type_name_comment() once per tag, cache the results.
    // Each call does O(n) brace-counting, and we'd otherwise call it 4x per tag.
    let parsed: Vec<_> = param_tags
        .iter()
        .map(|(tag, _)| {
            let (type_part, name_part, _) = tag.type_name_comment();
            (type_part.is_some(), name_part.map(|n| n.parsed()))
        })
        .collect();

    // Check that ALL @param tags have type annotations and names
    if !parsed.iter().all(|(has_type, name)| *has_type && name.is_some()) {
        return;
    }

    // Extract the cached names (we verified all are Some above)
    let names: Vec<&str> = parsed.iter().map(|(_, name)| name.unwrap_or("")).collect();

    // Extract function parameter names from the source text after the comment
    let fn_params = extract_function_params(comment, source_text);
    if fn_params.len() != names.len() {
        return;
    }

    // Already in order?
    if names.iter().zip(fn_params.iter()).all(|(name, p)| *name == *p) {
        return;
    }

    // Check same set of names (lengths already verified equal, param lists are small)
    if !names.iter().all(|name| fn_params.contains(name)) {
        return;
    }

    // Sort @param tags by their position in the function signature.
    // Use sort_by_cached_key to call the key function once per element.
    effective_tags[param_start..param_end].sort_by_cached_key(|(tag, _)| {
        let (_, name_part, _) = tag.type_name_comment();
        let name = name_part.map_or("", |n| n.parsed());
        fn_params.iter().position(|p| *p == name).unwrap_or(usize::MAX)
    });
}

/// Extract function parameter names from the source text after the comment.
/// Handles `function name(...)`, `name(...)` methods, `name = (...) =>` arrows.
/// Uses balanced parenthesis matching to handle nested type annotations.
fn extract_function_params<'a>(comment: &Comment, source_text: &'a str) -> Vec<&'a str> {
    let after_start = comment.span.end as usize;
    let after = &source_text[after_start..];

    // Find a function-like construct: look for identifier followed by `(`
    // Skip whitespace, look for `function`, `async`, method names, arrow patterns
    let trimmed = after.trim_start();

    // Find the opening `(` of the parameter list.
    // We look for patterns that indicate a function definition (not a call).
    let paren_pos = find_function_params_start(trimmed);
    let Some(paren_start) = paren_pos else {
        return Vec::new();
    };

    // Find matching closing `)` with balanced parenthesis counting
    let Some(paren_end) = find_matching_paren(trimmed, paren_start) else {
        return Vec::new();
    };

    let params_str = &trimmed[paren_start + 1..paren_end];

    // Parse parameter names, handling TypeScript type annotations
    parse_param_names(params_str)
}

/// Find the start position of function parameter parentheses in the text.
/// Returns the index of `(` in function-like constructs.
fn find_function_params_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    // Skip `export`, `async`, `default` keywords
    loop {
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let is_id_continue = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
        if text[i..].starts_with("export") && i + 6 < len && !is_id_continue(bytes[i + 6]) {
            i += 6;
            continue;
        }
        if text[i..].starts_with("async") && i + 5 < len && !is_id_continue(bytes[i + 5]) {
            i += 5;
            continue;
        }
        if text[i..].starts_with("default") && i + 7 < len && !is_id_continue(bytes[i + 7]) {
            i += 7;
            continue;
        }
        break;
    }

    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    // `function name(`
    if text[i..].starts_with("function") {
        i += 8;
        // Skip optional `*` for generators
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len && bytes[i] == b'*' {
            i += 1;
        }
        // Skip function name
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        // Skip TypeScript generics `<T>`
        if i < len
            && bytes[i] == b'<'
            && let Some(end) = find_matching_angle(text, i)
        {
            i = end + 1;
        }
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len && bytes[i] == b'(' {
            return Some(i);
        }
        return None;
    }

    // `const name = (` or `name(` (method)
    if i < len && (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' || bytes[i] == b'$') {
        // Skip `const`/`let`/`var` keyword
        if text[i..].starts_with("const ")
            || text[i..].starts_with("let ")
            || text[i..].starts_with("var ")
        {
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }

        // Skip identifier
        let id_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        if i == id_start {
            return None;
        }

        // Skip TypeScript generics
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < len
            && bytes[i] == b'<'
            && let Some(end) = find_matching_angle(text, i)
        {
            i = end + 1;
        }

        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        // Direct method: `name(`
        if i < len && bytes[i] == b'(' {
            return Some(i);
        }

        // Arrow: `name = (`
        if i < len && bytes[i] == b'=' && i + 1 < len && bytes[i + 1] != b'=' {
            i += 1;
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            // Skip `async`
            let is_id_continue = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
            if text[i..].starts_with("async") && i + 5 < len && !is_id_continue(bytes[i + 5]) {
                i += 5;
                while i < len && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
            if i < len && bytes[i] == b'(' {
                return Some(i);
            }
        }
    }

    None
}

/// Find matching closing angle bracket `>` for TypeScript generics.
fn find_matching_angle(text: &str, start: usize) -> Option<usize> {
    find_matching_angle_bytes(text.as_bytes(), start)
}

/// Advance `i` past the current parameter (type annotation, default value, etc.)
/// until the next comma delimiter, handling balanced parens, angles, and string literals.
fn skip_to_comma(bytes: &[u8], len: usize, i: &mut usize) {
    while *i < len && bytes[*i] != b',' {
        match bytes[*i] {
            b'(' => {
                if let Some(end) = find_matching_paren_bytes(bytes, *i) {
                    *i = end + 1;
                } else {
                    *i += 1;
                }
            }
            b'<' => {
                if let Some(end) = find_matching_angle_bytes(bytes, *i) {
                    *i = end + 1;
                } else {
                    *i += 1;
                }
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[*i];
                *i += 1;
                while *i < len && bytes[*i] != quote {
                    if bytes[*i] == b'\\' {
                        *i += 1;
                    }
                    *i += 1;
                }
                if *i < len {
                    *i += 1;
                }
            }
            _ => *i += 1,
        }
    }
}

/// Find matching closing angle bracket `>`, operating on raw bytes.
fn find_matching_angle_bytes(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find matching closing `)`, operating on raw bytes.
fn find_matching_paren_bytes(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'\'' | b'"' | b'`' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find matching closing `)` given position of opening `(`.
fn find_matching_paren(text: &str, start: usize) -> Option<usize> {
    find_matching_paren_bytes(text.as_bytes(), start)
}

/// Parse parameter names from a function parameter list string.
/// Handles TypeScript type annotations, default values, destructuring, and rest params.
fn parse_param_names(params_str: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut i = 0;
    let bytes = params_str.as_bytes();
    let len = bytes.len();

    while i < len {
        // Skip whitespace
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        // Handle destructuring — skip the whole `{...}` or `[...]` structure.
        // Destructured params have no single name to match against @param tags,
        // so we skip them entirely. The reorder will bail out at the length
        // check above because the extracted fn_params won't include an entry
        // for the destructured position, making the counts differ.
        if bytes[i] == b'{' || bytes[i] == b'[' {
            let (open, close) = if bytes[i] == b'{' { (b'{', b'}') } else { (b'[', b']') };
            let mut depth = 0;
            while i < len {
                if bytes[i] == open {
                    depth += 1;
                } else if bytes[i] == close {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            // Skip type annotation, default value, and comma (bracket-aware)
            skip_to_comma(bytes, len, &mut i);
            if i < len {
                i += 1; // skip comma
            }
            continue;
        }

        // Handle rest params: `...name`
        if i + 2 < len && bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
            i += 3;
        }

        // Extract parameter name
        let name_start = i;
        while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
        {
            i += 1;
        }
        if i > name_start {
            names.push(&params_str[name_start..i]);
        }

        // Skip type annotation (`: Type`), which may include nested parens/angles
        skip_to_comma(bytes, len, &mut i);
        if i < len {
            i += 1; // skip comma
        }
    }

    names
}

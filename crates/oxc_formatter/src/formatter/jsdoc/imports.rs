use super::{line_buffer::LineBuffer, serialize::join_iter};

/// A parsed `@import` tag.
#[derive(Clone)]
struct ImportInfo {
    default_import: Option<String>,
    named_imports: Vec<String>,
    module_path: String,
}

/// Parse an `@import` tag's comment text into its components.
///
/// Handles these forms:
/// - `Default, {Named1, Named2} from "module"`
/// - `{Named1} from 'module'`
/// - `Default from "module"`
fn parse_import_tag(comment_text: &str) -> Option<ImportInfo> {
    // Normalize: join lines, collapse whitespace
    let text: String = join_iter(comment_text.lines().map(str::trim), " ");
    let text = text.trim();

    // Find "from" keyword followed by a quoted string
    let from_idx = text.rfind(" from ")?;
    let specifier = text[..from_idx].trim();
    let module_part = text[from_idx + 6..].trim();

    // Extract module path (strip matching quotes)
    let quote = match module_part.as_bytes().first() {
        Some(b'"' | b'\'') => module_part.as_bytes()[0] as char,
        _ => return None,
    };
    let module_path = module_part.strip_prefix(quote)?.strip_suffix(quote)?;
    if module_path.is_empty() {
        return None;
    }

    // Parse specifier: "Default, {Named1, Named2}", "{Named1}", or "Default"
    let (default_import, named_imports) = if let Some(brace_start) = specifier.find('{') {
        let brace_end = specifier.rfind('}')?;
        let default_part = specifier[..brace_start].trim().trim_end_matches(',').trim();
        let named_part = &specifier[brace_start + 1..brace_end];

        let default_import =
            if default_part.is_empty() { None } else { Some(default_part.to_string()) };

        let named_imports: Vec<String> = named_part
            .split(',')
            .map(|s| {
                // Normalize whitespace: "B  as  B1" → "B as B1"
                join_iter(s.split_whitespace(), " ")
            })
            .filter(|s| !s.is_empty())
            .collect();

        (default_import, named_imports)
    } else {
        // No braces — just a default import
        let name = join_iter(specifier.split_whitespace(), " ");
        (Some(name), Vec::new())
    };

    Some(ImportInfo { default_import, named_imports, module_path: module_path.to_string() })
}

/// Case-insensitive ASCII string comparison (no allocation).
fn cmp_ascii_case_insensitive(a: &str, b: &str) -> std::cmp::Ordering {
    for (ca, cb) in a.bytes().zip(b.bytes()) {
        let ord = ca.to_ascii_lowercase().cmp(&cb.to_ascii_lowercase());
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    a.len().cmp(&b.len())
}

/// Get the sort key for a named import specifier (sort by alias).
/// `"B as B1"` → `"B1"`, `"B2"` → `"B2"`.
fn import_specifier_sort_key(specifier: &str) -> &str {
    if let Some(idx) = specifier.find(" as ") {
        specifier[idx + 4..].trim()
    } else {
        specifier.trim()
    }
}

/// Merge `@import` tags that share the same module path.
/// Returns merged imports sorted by module path (third-party before relative).
fn merge_and_sort_imports(imports: Vec<ImportInfo>) -> Vec<ImportInfo> {
    if imports.is_empty() {
        return imports;
    }

    // Group by module path (preserving insertion order)
    let mut groups: Vec<ImportInfo> = Vec::new();

    for import in imports {
        if let Some(existing) = groups.iter_mut().find(|g| g.module_path == import.module_path) {
            // Merge: take last default import, combine named imports
            if import.default_import.is_some() {
                existing.default_import = import.default_import;
            }
            for named in import.named_imports {
                // Deduplicate by original import name
                let key = import_specifier_sort_key(&named);
                let already_exists =
                    existing.named_imports.iter().any(|n| import_specifier_sort_key(n) == key);
                if !already_exists {
                    existing.named_imports.push(named);
                }
            }
        } else {
            groups.push(import);
        }
    }

    // Sort named imports within each group by original import name (case-insensitive)
    for import in &mut groups {
        import.named_imports.sort_by(|a, b| {
            cmp_ascii_case_insensitive(import_specifier_sort_key(a), import_specifier_sort_key(b))
        });
    }

    // Sort groups: third-party (no ./ or ../) before relative, then alphabetically
    groups.sort_by(|a, b| {
        let a_relative = a.module_path.starts_with('.');
        let b_relative = b.module_path.starts_with('.');
        match (a_relative, b_relative) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => cmp_ascii_case_insensitive(&a.module_path, &b.module_path),
        }
    });

    groups
}

/// Format a single merged `@import` tag into output lines.
fn format_import_lines(import: &ImportInfo, content_lines: &mut LineBuffer) {
    let module = &import.module_path;

    match (&import.default_import, import.named_imports.len()) {
        (Some(default), 0) => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(" from \"");
            s.push_str(module);
            s.push('"');
        }
        (None, 1) => {
            let s = content_lines.begin_line();
            s.push_str("@import {");
            s.push_str(&import.named_imports[0]);
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (Some(default), 1) => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(", {");
            s.push_str(&import.named_imports[0]);
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (None, n) if n >= 2 => {
            content_lines.push("@import {");
            for (i, named) in import.named_imports.iter().enumerate() {
                let s = content_lines.begin_line();
                s.push_str("  ");
                s.push_str(named);
                if i < import.named_imports.len() - 1 {
                    s.push(',');
                }
            }
            let s = content_lines.begin_line();
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        (Some(default), n) if n >= 2 => {
            let s = content_lines.begin_line();
            s.push_str("@import ");
            s.push_str(default);
            s.push_str(", {");
            for (i, named) in import.named_imports.iter().enumerate() {
                let s = content_lines.begin_line();
                s.push_str("  ");
                s.push_str(named);
                if i < import.named_imports.len() - 1 {
                    s.push(',');
                }
            }
            let s = content_lines.begin_line();
            s.push_str("} from \"");
            s.push_str(module);
            s.push('"');
        }
        _ => {}
    }
}

/// Process all `@import` tags: parse, merge by module, sort, and format.
/// Returns formatted lines ready to be inserted into the comment, plus
/// the set of tag indices that were successfully parsed (so unparsable
/// `@import` tags can fall through to `format_generic_tag()`).
pub(super) fn process_import_tags(
    tags: &[(&oxc_jsdoc::parser::JSDocTag<'_>, &str)],
) -> (LineBuffer, smallvec::SmallVec<[usize; 4]>) {
    let mut imports = Vec::new();
    let mut parsed_indices = smallvec::SmallVec::<[usize; 4]>::new();

    for (idx, &(tag, kind)) in tags.iter().enumerate() {
        if kind != "import" {
            continue;
        }
        let comment = tag.comment().parsed();
        if let Some(info) = parse_import_tag(&comment) {
            imports.push(info);
            parsed_indices.push(idx);
        }
    }

    let merged = merge_and_sort_imports(imports);

    let mut lines = LineBuffer::new();
    for import in &merged {
        format_import_lines(import, &mut lines);
    }
    (lines, parsed_indices)
}

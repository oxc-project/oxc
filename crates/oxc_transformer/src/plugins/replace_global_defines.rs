use std::{cmp::Ordering, sync::Arc};

use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_diagnostics::OxcDiagnostic;
use oxc_parser::Parser;
use oxc_semantic::{IsGlobalReference, ScopeTree, SymbolTable};
use oxc_span::{CompactStr, SourceType};
use oxc_syntax::identifier::is_identifier_name;
use oxc_traverse::{traverse_mut, Traverse, TraverseCtx};

/// Configuration for [ReplaceGlobalDefines].
///
/// Due to the usage of an arena allocator, the constructor will parse once for grammatical errors,
/// and does not save the constructed expression.
///
/// The data is stored in an `Arc` so this can be shared across threads.
#[derive(Debug, Clone)]
pub struct ReplaceGlobalDefinesConfig(Arc<ReplaceGlobalDefinesConfigImpl>);

#[derive(Debug)]
struct ReplaceGlobalDefinesConfigImpl {
    identifier: Vec<(/* key */ CompactStr, /* value */ CompactStr)>,
    dot: Vec<DotDefine>,
    meta_property: Vec<MetaPropertyDefine>,
    /// extra field to avoid linear scan `meta_property` to check if it has `import.meta` every
    /// time
    /// Some(replacement): import.meta -> replacement
    /// None -> no need to replace import.meta
    import_meta: Option<CompactStr>,
}

#[derive(Debug)]
pub struct DotDefine {
    /// Member expression parts
    pub parts: Vec<CompactStr>,
    pub value: CompactStr,
}

#[derive(Debug)]
pub struct MetaPropertyDefine {
    /// only store parts after `import.meta`
    pub parts: Vec<CompactStr>,
    pub value: CompactStr,
    pub postfix_wildcard: bool,
}

impl MetaPropertyDefine {
    pub fn new(parts: Vec<CompactStr>, value: CompactStr, postfix_wildcard: bool) -> Self {
        Self { parts, value, postfix_wildcard }
    }
}

impl DotDefine {
    fn new(parts: Vec<CompactStr>, value: CompactStr) -> Self {
        Self { parts, value }
    }
}

enum IdentifierType {
    Identifier,
    DotDefines { parts: Vec<CompactStr> },
    // import.meta.a
    ImportMetaWithParts { parts: Vec<CompactStr>, postfix_wildcard: bool },
    // import.meta or import.meta.*
    ImportMeta(bool),
}

impl ReplaceGlobalDefinesConfig {
    /// # Errors
    ///
    /// * key is not an identifier
    /// * value has a syntax error
    pub fn new<S: AsRef<str>>(defines: &[(S, S)]) -> Result<Self, Vec<OxcDiagnostic>> {
        let allocator = Allocator::default();
        let mut identifier_defines = vec![];
        let mut dot_defines = vec![];
        let mut meta_properties_defines = vec![];
        let mut import_meta = None;
        for (key, value) in defines {
            let key = key.as_ref();

            let value = value.as_ref();
            Self::check_value(&allocator, value)?;

            match Self::check_key(key)? {
                IdentifierType::Identifier => {
                    identifier_defines.push((CompactStr::new(key), CompactStr::new(value)));
                }
                IdentifierType::DotDefines { parts } => {
                    dot_defines.push(DotDefine::new(parts, CompactStr::new(value)));
                }
                IdentifierType::ImportMetaWithParts { parts, postfix_wildcard } => {
                    meta_properties_defines.push(MetaPropertyDefine::new(
                        parts,
                        CompactStr::new(value),
                        postfix_wildcard,
                    ));
                }
                IdentifierType::ImportMeta(postfix_wildcard) => {
                    if postfix_wildcard {
                        meta_properties_defines.push(MetaPropertyDefine::new(
                            vec![],
                            CompactStr::new(value),
                            postfix_wildcard,
                        ));
                    } else {
                        import_meta = Some(CompactStr::new(value));
                    }
                }
            }
        }
        // Always move specific meta define before wildcard dot define
        // Keep other order unchanged
        // see test case replace_global_definitions_dot_with_postfix_mixed as an example
        meta_properties_defines.sort_by(|a, b| {
            if !a.postfix_wildcard && b.postfix_wildcard {
                Ordering::Less
            } else if a.postfix_wildcard && b.postfix_wildcard {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        Ok(Self(Arc::new(ReplaceGlobalDefinesConfigImpl {
            identifier: identifier_defines,
            dot: dot_defines,
            meta_property: meta_properties_defines,
            import_meta,
        })))
    }

    fn check_key(key: &str) -> Result<IdentifierType, Vec<OxcDiagnostic>> {
        let parts: Vec<&str> = key.split('.').collect();

        assert!(!parts.is_empty());

        if parts.len() == 1 {
            if !is_identifier_name(parts[0]) {
                return Err(vec![OxcDiagnostic::error(format!("`{key}` is not an identifier."))]);
            }
            return Ok(IdentifierType::Identifier);
        }
        let normalized_parts_len =
            if parts[parts.len() - 1] == "*" { parts.len() - 1 } else { parts.len() };
        // We can ensure now the parts.len() >= 2
        let is_import_meta = parts[0] == "import" && parts[1] == "meta";

        for part in &parts[0..normalized_parts_len] {
            if !is_identifier_name(part) {
                return Err(vec![OxcDiagnostic::error(format!("`{key}` is not an identifier."))]);
            }
        }
        if is_import_meta {
            match normalized_parts_len {
                2 => Ok(IdentifierType::ImportMeta(normalized_parts_len != parts.len())),
                _ => Ok(IdentifierType::ImportMetaWithParts {
                    parts: parts
                        .iter()
                        .skip(2)
                        .take(normalized_parts_len - 2)
                        .map(|s| CompactStr::new(s))
                        .collect(),
                    postfix_wildcard: normalized_parts_len != parts.len(),
                }),
            }
        // StaticMemberExpression with postfix wildcard
        } else if normalized_parts_len != parts.len() {
            Err(vec![OxcDiagnostic::error(
                "postfix wildcard is only allowed for `import.meta`.".to_string(),
            )])
        } else {
            Ok(IdentifierType::DotDefines {
                parts: parts
                    .iter()
                    .take(normalized_parts_len)
                    .map(|s| CompactStr::new(s))
                    .collect(),
            })
        }
    }

    fn check_value(allocator: &Allocator, source_text: &str) -> Result<(), Vec<OxcDiagnostic>> {
        Parser::new(allocator, source_text, SourceType::default()).parse_expression()?;
        Ok(())
    }
}

#[must_use]
pub struct ReplaceGlobalDefinesReturn {
    pub symbols: SymbolTable,
    pub scopes: ScopeTree,
}

/// Replace Global Defines.
///
/// References:
///
/// * <https://esbuild.github.io/api/#define>
/// * <https://github.com/terser/terser?tab=readme-ov-file#conditional-compilation>
/// * <https://github.com/evanw/esbuild/blob/9c13ae1f06dfa909eb4a53882e3b7e4216a503fe/internal/config/globals.go#L852-L1014>
pub struct ReplaceGlobalDefines<'a> {
    allocator: &'a Allocator,
    config: ReplaceGlobalDefinesConfig,
}

impl<'a> Traverse<'a> for ReplaceGlobalDefines<'a> {
    fn enter_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        self.replace_identifier_defines(expr, ctx);
        self.replace_dot_defines(expr, ctx);
    }
}

impl<'a> ReplaceGlobalDefines<'a> {
    pub fn new(allocator: &'a Allocator, config: ReplaceGlobalDefinesConfig) -> Self {
        Self { allocator, config }
    }

    pub fn build(
        &mut self,
        symbols: SymbolTable,
        scopes: ScopeTree,
        program: &mut Program<'a>,
    ) -> ReplaceGlobalDefinesReturn {
        let (symbols, scopes) = traverse_mut(self, self.allocator, program, symbols, scopes);
        ReplaceGlobalDefinesReturn { symbols, scopes }
    }

    // Construct a new expression because we don't have ast clone right now.
    fn parse_value(&self, source_text: &str) -> Expression<'a> {
        // Allocate the string lazily because replacement happens rarely.
        let source_text = self.allocator.alloc_str(source_text);
        // Unwrapping here, it should already be checked by [ReplaceGlobalDefinesConfig::new].
        Parser::new(self.allocator, source_text, SourceType::default()).parse_expression().unwrap()
    }

    fn replace_identifier_defines(&self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        let Expression::Identifier(ident) = expr else { return };
        if !ident.is_global_reference(ctx.symbols()) {
            return;
        }
        for (key, value) in &self.config.0.identifier {
            if ident.name.as_str() == key {
                let value = self.parse_value(value);
                *expr = value;
                break;
            }
        }
    }

    fn replace_dot_defines(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a>) {
        match expr {
            Expression::StaticMemberExpression(member) => {
                for dot_define in &self.config.0.dot {
                    if Self::is_dot_define(ctx.symbols(), dot_define, member) {
                        let value = self.parse_value(&dot_define.value);
                        *expr = value;
                        return;
                    }
                }
                for meta_proeperty_define in &self.config.0.meta_property {
                    if Self::is_meta_property_define(meta_proeperty_define, member) {
                        let value = self.parse_value(&meta_proeperty_define.value);
                        *expr = value;
                        return;
                    }
                }
            }
            Expression::MetaProperty(meta_property) => {
                if let Some(ref replacement) = self.config.0.import_meta {
                    if meta_property.meta.name == "import" && meta_property.property.name == "meta"
                    {
                        let value = self.parse_value(replacement);
                        *expr = value;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn is_meta_property_define(
        meta_define: &MetaPropertyDefine,
        member: &StaticMemberExpression<'a>,
    ) -> bool {
        if meta_define.parts.is_empty() && meta_define.postfix_wildcard {
            match member.object {
                Expression::MetaProperty(ref meta) => {
                    return meta.meta.name == "import" && meta.property.name == "meta";
                }
                _ => return false,
            }
        }
        debug_assert!(!meta_define.parts.is_empty());

        let mut current_part_member_expression = Some(member);
        let mut cur_part_name = &member.property.name;
        let mut is_full_match = true;
        let mut i = meta_define.parts.len() - 1;
        let mut has_matched_part = false;
        loop {
            let part = &meta_define.parts[i];
            let matched = cur_part_name.as_str() == part;
            if matched {
                has_matched_part = true;
            } else {
                is_full_match = false;
                // Considering import.meta.env.*
                // ```js
                // import.meta.env.test // should matched
                // import.res.meta.env // should not matched
                // ```
                // So we use has_matched_part to track if any part has matched.

                if !meta_define.postfix_wildcard || has_matched_part {
                    return false;
                }
            }

            current_part_member_expression = if let Some(member) = current_part_member_expression {
                match &member.object {
                    Expression::StaticMemberExpression(member) => {
                        cur_part_name = &member.property.name;
                        Some(member)
                    }
                    Expression::MetaProperty(_) => {
                        if meta_define.postfix_wildcard {
                            // `import.meta.env` should not match `import.meta.env.*`
                            return has_matched_part && !is_full_match;
                        }
                        return true;
                    }
                    Expression::Identifier(_) => {
                        return false;
                    }
                    _ => None,
                }
            } else {
                return false;
            };

            // Config `import.meta.env.* -> 'undefined'`
            // Considering try replace `import.meta.env` to `undefined`, for the first loop the i is already
            // 0, if it did not match part name and still reach here, that means
            // current_part_member_expression is still something, and possible to match in the
            // further loop
            if i == 0 && matched {
                break;
            }

            if matched {
                i -= 1;
            }
        }

        false
    }

    pub fn is_dot_define(
        symbols: &SymbolTable,
        dot_define: &DotDefine,
        member: &StaticMemberExpression<'a>,
    ) -> bool {
        debug_assert!(dot_define.parts.len() > 1);

        let mut current_part_member_expression = Some(member);
        let mut cur_part_name = &member.property.name;

        for (i, part) in dot_define.parts.iter().enumerate().rev() {
            if cur_part_name.as_str() != part {
                return false;
            }

            if i == 0 {
                break;
            }

            current_part_member_expression = if let Some(member) = current_part_member_expression {
                match &member.object {
                    Expression::StaticMemberExpression(member) => {
                        cur_part_name = &member.property.name;
                        Some(member)
                    }
                    Expression::Identifier(ident) => {
                        if !ident.is_global_reference(symbols) {
                            return false;
                        }
                        cur_part_name = &ident.name;
                        None
                    }
                    _ => None,
                }
            } else {
                return false;
            };
        }

        true
    }
}

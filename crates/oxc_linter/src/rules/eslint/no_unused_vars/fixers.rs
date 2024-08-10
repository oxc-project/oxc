use oxc_ast::{
    ast::{BindingPatternKind, VariableDeclaration, VariableDeclarator},
    AstKind,
};
use oxc_semantic::{AstNode, AstNodeId};
use oxc_span::{CompactStr, GetSpan, Span};
use regex::Regex;

use super::{NoUnusedVars, Symbol};
use crate::fixer::{Fix, RuleFix, RuleFixer};

impl NoUnusedVars {
    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn rename_or_remove_var_declaration<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        decl: &VariableDeclarator<'a>,
        decl_id: AstNodeId,
    ) -> RuleFix<'a> {
        let Some(AstKind::VariableDeclaration(declaration)) =
            symbol.nodes().parent_node(decl_id).map(AstNode::kind)
        else {
            panic!("VariableDeclarator nodes should always be direct children of VariableDeclaration nodes");
        };

        // `true` even if references aren't considered a usage.
        let has_references = symbol.has_references();

        // we can delete variable declarations that aren't referenced anywhere
        if !has_references {
            // for `let x = 1;` or `const { x } = obj; the whole declaration can
            // be removed, but for `const { x, y } = obj;` or `let x = 1, y = 2`
            // we need to keep the other declarations
            let has_neighbors = declaration.declarations.len() > 1;
            debug_assert!(!declaration.declarations.is_empty());
            let binding_info = symbol.get_binding_info(&decl.id.kind);

            match binding_info {
                BindingInfo::SingleDestructure | BindingInfo::NotDestructure => {
                    if has_neighbors {
                        return self
                            .delete_declarator(fixer, symbol, declaration, decl)
                            .dangerously();
                    }
                    return fixer.delete(declaration).dangerously();
                }
                BindingInfo::MultiDestructure(mut span, is_object, is_last) => {
                    let source_after = &fixer.source_text()[(span.end as usize)..];
                    // remove trailing commas
                    span = span.expand_right(count_whitespace_or_commas(source_after.chars()));

                    // remove leading commas when removing the last element in
                    // an array
                    // `const [a, b] = [1, 2];` -> `const [a, b] = [1, 2];`
                    //            ^                         ^^^
                    if !is_object && is_last {
                        debug_assert!(span.start > 0);
                        let source_before = &fixer.source_text()[..(span.start as usize)];
                        let chars = source_before.chars().rev();
                        let start_offset = count_whitespace_or_commas(chars);
                        // do not walk past the beginning of the file
                        debug_assert!(start_offset < span.start);
                        span = span.expand_left(start_offset);
                    }

                    return if is_object || is_last {
                        fixer.delete_range(span).dangerously()
                    } else {
                        // infix array elements need a comma to preserve
                        // unpacking order of symbols around them
                        // e.g. `const [a, b, c] = [1, 2, 3];` -> `const [a, , c] = [1, 2, 3];`
                        fixer.replace(span, ",").dangerously()
                    };
                }
                BindingInfo::NotFound => {
                    return fixer.noop();
                }
            }
        }

        // otherwise, try to rename the variable to match the unused variable
        // pattern
        if let Some(new_name) = self.get_unused_var_name(symbol) {
            return symbol.rename(&new_name).dangerously();
        }

        fixer.noop()
    }

    /// Delete a single declarator from a [`VariableDeclaration`] list with more
    /// than one declarator.
    #[allow(clippy::unused_self)]
    fn delete_declarator<'a>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        symbol: &Symbol<'_, 'a>,
        declaration: &VariableDeclaration<'a>,
        decl: &VariableDeclarator<'a>,
    ) -> RuleFix<'a> {
        let own_position = declaration
            .declarations
            .iter()
            .position(|d| symbol == &d.id)
            .expect("VariableDeclarator not found within its own parent VariableDeclaration");
        let mut delete_range = decl.span();
        let mut has_left = false;
        let mut has_right = false;

        // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
        //             ^^^^^                         ^^^^^^^
        if let Some(right_neighbor) = declaration.declarations.get(own_position + 1) {
            delete_range.end = right_neighbor.span.start;
            has_right = true;
        }

        // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
        //             ^^^^^                       ^^^^^^^
        if own_position > 0 {
            if let Some(left_neighbor) = declaration.declarations.get(own_position - 1) {
                delete_range.start = left_neighbor.span.end;
                has_left = true;
            }
        }

        // both left and right neighbors are present, so we need to
        // re-insert a comma
        // `let x = 1, y = 2, z = 3;`
        //           ^^^^^^^^^
        if has_left && has_right {
            return fixer.replace(delete_range, ", ");
        }

        return fixer.delete(&delete_range);
    }

    fn get_unused_var_name(&self, symbol: &Symbol<'_, '_>) -> Option<CompactStr> {
        let pat = self.vars_ignore_pattern.as_ref().map(Regex::as_str)?;

        let ignored_name: String = match pat {
            // TODO: support more patterns
            "^_" => format!("_{}", symbol.name()),
            _ => return None,
        };

        // adjust name to avoid conflicts
        let scopes = symbol.scopes();
        let scope_id = symbol.scope_id();
        let mut i = 0;
        let mut new_name = ignored_name.clone();
        while scopes.has_binding(scope_id, &new_name) {
            new_name = format!("{ignored_name}{i}");
            i += 1;
        }

        Some(new_name.into())
    }
}

impl<'s, 'a> Symbol<'s, 'a> {
    fn rename(&self, new_name: &CompactStr) -> RuleFix<'a> {
        let mut fixes: Vec<Fix<'a>> = vec![];
        let decl_span = self.span();
        fixes.push(Fix::new(new_name.clone(), decl_span));

        for reference in self.references() {
            match self.nodes().get_node(reference.node_id()).kind() {
                AstKind::IdentifierReference(id) => {
                    fixes.push(Fix::new(new_name.clone(), id.span()));
                }
                AstKind::TSTypeReference(ty) => {
                    fixes.push(Fix::new(new_name.clone(), ty.type_name.span()));
                }
                // we found a reference to an unknown node and we don't know how
                // to replace it, so we abort the whole process
                _ => return Fix::empty().into(),
            }
        }

        return RuleFix::from(fixes)
            .with_message(format!("Rename '{}' to '{new_name}'", self.name()));
    }

    /// - `true` if `pattern` is a destructuring pattern and only contains one symbol
    /// - `false` if `pattern` is a destructuring pattern and contains more than one symbol
    /// - `not applicable` if `pattern` is not a destructuring pattern
    fn get_binding_info(&self, pattern: &BindingPatternKind<'a>) -> BindingInfo {
        match pattern {
            BindingPatternKind::ArrayPattern(arr) => match arr.elements.len() {
                0 => {
                    debug_assert!(arr.rest.is_some());

                    BindingInfo::multi_or_single(arr.rest.as_ref().map(|r| (r.span, true)), true)
                }
                1 => {
                    let own_span =
                        arr.elements.iter().filter_map(|el| el.as_ref()).find(|el| self == *el);
                    if let Some(rest) = arr.rest.as_ref() {
                        if rest.span.contains_inclusive(self.span()) {
                            // spreads are after all elements, otherwise it
                            // would be a spread element
                            return BindingInfo::MultiDestructure(rest.span, false, true);
                        }

                        BindingInfo::multi_or_missing(own_span.map(|el| (el.span(), false)), false)
                    } else {
                        BindingInfo::single_or_missing(own_span.is_some())
                    }
                }
                _ => {
                    let last_position = arr.elements.len() - 1;
                    BindingInfo::multi_or_missing(
                        arr.elements
                            .iter()
                            .enumerate()
                            .filter_map(|(idx, el)| el.as_ref().map(|el| (idx, el)))
                            .find_map(|(idx, el)| {
                                if self == el {
                                    Some((el.span(), idx == last_position))
                                } else {
                                    None
                                }
                            }),
                        false,
                    )
                }
            },
            BindingPatternKind::ObjectPattern(obj) => match obj.properties.len() {
                0 => {
                    debug_assert!(obj.rest.is_some());
                    BindingInfo::multi_or_single(obj.rest.as_ref().map(|r| (r.span, true)), true)
                }
                1 => {
                    let last_property = obj.properties.len() - 1;
                    let own_span = obj.properties.iter().enumerate().find_map(|(idx, el)| {
                        if self == &el.value {
                            Some((el.span, idx == last_property))
                        } else {
                            None
                        }
                    });
                    if let Some(rest) = obj.rest.as_ref() {
                        if rest.span.contains_inclusive(self.span()) {
                            // assume rest spreading in objects are at the end
                            return BindingInfo::MultiDestructure(rest.span, true, true);
                        }
                        BindingInfo::multi_or_missing(own_span, true)
                    } else {
                        BindingInfo::single_or_missing(own_span.is_some())
                    }
                }
                _ => {
                    let last_property = obj.properties.len() - 1;
                    let own_span = obj.properties.iter().enumerate().find_map(|(idx, el)| {
                        (self == &el.value).then_some((el.span, idx == last_property))
                    });
                    BindingInfo::multi_or_missing(own_span, true)
                }
            },
            BindingPatternKind::AssignmentPattern(assignment) => {
                self.get_binding_info(&assignment.left.kind)
            }
            // not in a destructure
            BindingPatternKind::BindingIdentifier(_) => BindingInfo::NotDestructure,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum BindingInfo {
    NotDestructure,
    SingleDestructure,
    /// Notes:
    /// 1. Symbol declaration span will cover the entire pattern, so we need
    ///    to extract the symbol's _exact_ span
    /// 2. Unused symbols created in array destructures need to have a comma if
    ///    they are not in the last position. The second boolean arg indicates
    ///    if the pattern is an object destructure (true) or an array
    ///    destructure (false). When the symbol is in the last position, it
    ///    doesn't need a comma, which is what the third boolean arg indicates.
    ///    It is not used for objects.
    MultiDestructure(Span, /* object? */ bool, /* last? */ bool),
    /// Usually indicates a problem in the AST, though it's possible for it to
    /// be a problem in the fixer
    NotFound,
}

impl BindingInfo {
    #[inline]
    const fn single_or_missing(found: bool) -> Self {
        if found {
            BindingInfo::SingleDestructure
        } else {
            BindingInfo::NotFound
        }
    }

    fn multi_or_missing(found: Option<(Span, bool)>, is_object: bool) -> Self {
        match found {
            Some((span, is_last)) => BindingInfo::MultiDestructure(span.span(), is_object, is_last),
            None => BindingInfo::NotFound,
        }
    }

    fn multi_or_single(found: Option<(Span, bool)>, is_object: bool) -> Self {
        match found {
            Some((span, is_last)) => BindingInfo::MultiDestructure(span.span(), is_object, is_last),
            None => BindingInfo::SingleDestructure,
        }
    }
}

// source text will never be large enough for this usize to be truncated when
// getting cast to a u32
#[allow(clippy::cast_possible_truncation)]
fn count_whitespace_or_commas<I: Iterator<Item = char>>(iter: I) -> u32 {
    iter.take_while(|c| c.is_whitespace() || *c == ',').count() as u32
}

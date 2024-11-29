use oxc_ast::{ast::*, AstKind};
use oxc_span::{CompactStr, GetSpan, Span};

use super::Symbol;
use crate::fixer::{Fix, RuleFix, RuleFixer};

impl<'s, 'a> Symbol<'s, 'a> {
    /// Delete a single declarator from a [`VariableDeclaration`] list with more
    /// than one declarator.
    #[allow(clippy::unused_self)]
    pub(super) fn delete_from_list<T>(
        &self,
        fixer: RuleFixer<'_, 'a>,
        list: &[T],
        own: &T,
    ) -> RuleFix<'a>
    where
        T: GetSpan,
        Symbol<'s, 'a>: PartialEq<T>,
    {
        let Some(own_position) = list.iter().position(|el| self == el) else {
            // Happens when the symbol is in a destructuring pattern.
            return fixer.noop();
        };
        let mut delete_range = own.span();
        let mut has_left = false;
        let mut has_right = false;

        // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
        //             ^^^^^                         ^^^^^^^
        if let Some(right_neighbor) = list.get(own_position + 1) {
            delete_range.end = right_neighbor.span().start;
            has_right = true;
        }

        // `let x = 1, y = 2, z = 3;` -> `let x = 1, y = 2, z = 3;`
        //             ^^^^^                       ^^^^^^^
        if own_position > 0 {
            if let Some(left_neighbor) = list.get(own_position - 1) {
                delete_range.start = left_neighbor.span().end;
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

        fixer.delete(&delete_range)
    }

    pub(super) fn rename(&self, new_name: &CompactStr) -> RuleFix<'a> {
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

        RuleFix::from(fixes).with_message(format!("Rename '{}' to '{new_name}'", self.name()))
    }

    /// - `true` if `pattern` is a destructuring pattern and only contains one symbol
    /// - `false` if `pattern` is a destructuring pattern and contains more than one symbol
    /// - `not applicable` if `pattern` is not a destructuring pattern
    pub(super) fn get_binding_info(&self, pattern: &BindingPatternKind<'a>) -> BindingInfo {
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
pub(super) enum BindingInfo {
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

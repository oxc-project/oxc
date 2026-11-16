use oxc_allocator::{Allocator, ArenaVec};
use oxc_data_structures::inline_string::InlineString;
use oxc_semantic::{LabelId, Scoping};
use oxc_str::{ArenaIdentHashSet, Ident};
use oxc_syntax::keyword::is_reserved_keyword;

/// Rename every label in `scoping` by its nesting depth.
///
/// Labels have their own namespace, so the names only need to avoid reserved keywords and the
/// names of unresolved `break` and `continue` labels, which codegen prints unchanged.
/// Labels in the same nesting chain have distinct depths and therefore distinct names,
/// while sibling labels reuse names.
pub fn mangle_labels<const CAPACITY: usize>(
    allocator: &Allocator,
    scoping: &mut Scoping,
    generate_name: impl Fn(u32) -> InlineString<CAPACITY, u8>,
) {
    let labels_len = scoping.labels_len();
    if labels_len == 0 {
        return;
    }

    let unresolved_names = ArenaIdentHashSet::from_iter_in(
        scoping.unresolved_label_names().map(|name| Ident::from(allocator.alloc_str(name))),
        allocator,
    );
    let mut depths = ArenaVec::with_capacity_in(labels_len, &allocator);
    let mut names: ArenaVec<Ident<'_>> = ArenaVec::new_in(&allocator);
    let mut next_candidate = 0;
    for label_id in (0..labels_len).map(LabelId::from_usize) {
        let depth =
            scoping.label_parent_id(label_id).map_or(0, |parent_id| depths[parent_id.index()] + 1);
        depths.push(depth);

        while names.len() <= depth {
            let name = generate_name(next_candidate);
            next_candidate += 1;
            if is_reserved_keyword(name.as_str()) {
                continue;
            }
            let name = Ident::from(allocator.alloc_str(name.as_str()));
            if !unresolved_names.contains(&name) {
                names.push(name);
            }
        }
        scoping.set_label_name(label_id, names[depth]);
    }
}

#[cfg(test)]
mod test {
    use rustc_hash::FxHashSet;

    use oxc_allocator::Allocator;
    use oxc_semantic::Scoping;
    use oxc_syntax::keyword::is_reserved_keyword;

    use super::mangle_labels;
    use crate::base54;

    #[test]
    fn nested_label_names() {
        let mut scoping = Scoping::default();
        let mut parent_id = None;
        let mut label_ids = Vec::with_capacity(4000);
        for _ in 0..4000 {
            let label_id = scoping.create_label("label".into(), parent_id);
            parent_id = Some(label_id);
            label_ids.push(label_id);
        }

        mangle_labels(&Allocator::default(), &mut scoping, base54);

        let names =
            label_ids.iter().map(|&label_id| scoping.label_name(label_id)).collect::<Vec<_>>();
        assert_eq!(names[..3], ["e", "t", "n"]);
        assert!(names.iter().all(|name| !is_reserved_keyword(name)));
        assert_eq!(names.iter().collect::<FxHashSet<_>>().len(), names.len());
    }

    #[test]
    fn sibling_label_names() {
        let mut scoping = Scoping::default();
        let outer = scoping.create_label("outer".into(), None);
        let first = scoping.create_label("first".into(), Some(outer));
        let second = scoping.create_label("second".into(), Some(outer));
        let next = scoping.create_label("next".into(), None);

        mangle_labels(&Allocator::default(), &mut scoping, base54);

        assert_eq!(
            [outer, first, second, next].map(|label_id| scoping.label_name(label_id)),
            ["e", "t", "t", "e"]
        );
    }

    #[test]
    fn unresolved_label_names() {
        let mut scoping = Scoping::default();
        let outer = scoping.create_label("outer".into(), None);
        let inner = scoping.create_label("inner".into(), Some(outer));
        scoping.add_unresolved_label_name("e".into());

        mangle_labels(&Allocator::default(), &mut scoping, base54);

        assert_eq!([outer, inner].map(|label_id| scoping.label_name(label_id)), ["t", "n"]);
    }
}

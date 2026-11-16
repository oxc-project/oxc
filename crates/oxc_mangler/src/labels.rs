use oxc_allocator::{Allocator, ArenaVec};
use oxc_data_structures::inline_string::InlineString;
use oxc_semantic::{LabelId, Scoping};
use oxc_syntax::keyword::is_reserved_keyword;

/// Rename every label in `scoping` by its nesting depth.
///
/// Labels have their own namespace, so the names only need to avoid reserved keywords.
/// Labels in the same nesting chain have distinct depths and therefore distinct names,
/// while sibling labels reuse names.
///
/// Codegen prints unresolved `break` and `continue` labels unchanged, and a renamed label could
/// capture one of them, so labels are kept when any exist. That only happens in invalid code.
pub fn mangle_labels<const CAPACITY: usize>(
    allocator: &Allocator,
    scoping: &mut Scoping,
    generate_name: impl Fn(u32) -> InlineString<CAPACITY, u8>,
) {
    if scoping.has_unresolved_labels() {
        return;
    }
    let Some(max_depth) = scoping.label_ids().map(|label_id| scoping.label_depth(label_id)).max()
    else {
        return;
    };
    let names = ArenaVec::from_iter_in(
        (0..)
            .map(generate_name)
            .filter(|name| !is_reserved_keyword(name.as_str()))
            .take(max_depth as usize + 1),
        &allocator,
    );
    for label_id in (0..scoping.labels_len()).map(LabelId::from_usize) {
        let name = &names[scoping.label_depth(label_id) as usize];
        scoping.set_label_name(label_id, name.as_str().into());
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
        let label_ids =
            (0..4000).map(|depth| scoping.create_label("label".into(), depth)).collect::<Vec<_>>();

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
        let outer = scoping.create_label("outer".into(), 0);
        let first = scoping.create_label("first".into(), 1);
        let second = scoping.create_label("second".into(), 1);
        let next = scoping.create_label("next".into(), 0);

        mangle_labels(&Allocator::default(), &mut scoping, base54);

        assert_eq!(
            [outer, first, second, next].map(|label_id| scoping.label_name(label_id)),
            ["e", "t", "t", "e"]
        );
    }

    #[test]
    fn unresolved_labels_keep_names() {
        let mut scoping = Scoping::default();
        let outer = scoping.create_label("outer".into(), 0);
        let inner = scoping.create_label("inner".into(), 1);
        scoping.set_has_unresolved_labels();

        mangle_labels(&Allocator::default(), &mut scoping, base54);

        assert_eq!([outer, inner].map(|label_id| scoping.label_name(label_id)), ["outer", "inner"]);
    }
}

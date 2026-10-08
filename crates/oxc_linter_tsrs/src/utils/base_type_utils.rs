//! Ports of internal/utils/base_type_utils.go helpers, added as rules need them.

use super::*;

use tsrs_checker::ObjectFlags;

/// base_type_utils.go MatchesTypeOrBaseType.
pub fn matches_type_or_base_type(
    c: &mut Checker,
    t: P<Type>,
    predicate: &mut dyn FnMut(&mut Checker, P<Type>) -> bool,
) -> bool {
    if predicate(c, t) {
        return true;
    }
    if !is_object_type(t) {
        return false;
    }
    let mut target = t;
    if t.object_flags().intersects(ObjectFlags::Reference) {
        target = t.target().unwrap();
    }
    if target.object_flags().intersects(ObjectFlags::ClassOrInterface) {
        for &base_type in c.get_base_types(target) {
            if matches_type_or_base_type(c, base_type, predicate) {
                return true;
            }
        }
    }
    false
}

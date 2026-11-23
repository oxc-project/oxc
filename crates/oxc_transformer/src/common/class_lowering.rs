//! Per-class ownership of class lowering.
//!
//! Transforms look up the lowering decision by class scope. Decisions that depend on decorators
//! or auto-accessors are recorded at class entry, before any transform can change the class.

use oxc_ast::ast::{Class, ClassElement};
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    decorator::DecoratorMode,
    utils::decorated_class::{
        class_body_has_auto_accessors, class_has_decorators, is_auto_accessor,
        legacy_decorators_reassign_class_binding,
    },
};

use super::class_transform::ClassTransform;

/// Which transform owns a class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ClassLoweringMode {
    /// Class fields stay native; auto-accessors and static blocks may still be lowered.
    Native,
    /// Class-properties lowers this class because the target requires class field lowering.
    LowerAll,
    /// The class keeps its decorators and auto-accessors for a downstream transform.
    Preserve,
}

/// Ownership of one class, decided from its untransformed AST.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClassLowering {
    mode: ClassLoweringMode,
    has_decorators: bool,
    has_auto_accessor: bool,
    /// Legacy decorators reassign the class binding after the class is defined.
    class_binding_is_reassigned: bool,
}

impl ClassLowering {
    fn compute(class_transform: ClassTransform, class: &Class<'_>) -> Self {
        let has_decorators = class_has_decorators(class);
        let has_auto_accessor = class_body_has_auto_accessors(&class.body);
        // Classes with decorators or auto-accessors are preserved for a downstream transform.
        let mode = if has_decorators || has_auto_accessor {
            ClassLoweringMode::Preserve
        } else {
            match class_transform {
                ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
                ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
            }
        };
        Self { mode, has_decorators, has_auto_accessor, class_binding_is_reassigned: false }
    }

    /// Whether class-properties lowers the fields of this class.
    pub fn lowers_fields(self) -> bool {
        self.mode == ClassLoweringMode::LowerAll
    }

    /// Whether a field or auto-accessor stays as written for the downstream transform.
    ///
    /// A preserved class keeps its decorated fields and its auto-accessors. Transforms that
    /// rewrite the other fields must leave these in place, and must not move other initializers
    /// after theirs (see [`Self::first_movable_instance_field`]).
    pub fn keeps_field(self, element: &ClassElement<'_>) -> bool {
        self.mode == ClassLoweringMode::Preserve
            && (is_auto_accessor(element)
                || matches!(element, ClassElement::PropertyDefinition(_))
                    && element.has_decorator())
    }

    /// Index of the first instance field whose initializer can move after the field
    /// initializers that stay in the class, such as into the constructor.
    ///
    /// Instance fields before the last kept instance field that runs code on initialization (a
    /// decorated field, or an auto-accessor with an initializer or decorators) must stay in
    /// place, or their initializers would run after it.
    pub fn first_movable_instance_field(self, class: &Class<'_>) -> usize {
        if self.mode != ClassLoweringMode::Preserve {
            return 0;
        }
        class
            .body
            .body
            .iter()
            .rposition(|element| {
                !element.r#static()
                    && self.keeps_field(element)
                    && match element {
                        ClassElement::AccessorProperty(accessor) => {
                            accessor.value.is_some() || element.has_decorator()
                        }
                        _ => true,
                    }
            })
            .map_or(0, |index| index + 1)
    }

    /// Whether the class or one of its elements is decorated.
    pub fn has_decorators(self) -> bool {
        self.has_decorators
    }

    pub fn has_auto_accessor(self) -> bool {
        self.has_auto_accessor
    }

    /// Whether legacy decorators reassign the class binding to the decorated class.
    ///
    /// e.g. `Foo` in `let Foo = class Foo {}; Foo = _decorate([dec], Foo);`
    pub fn class_binding_is_reassigned(self) -> bool {
        self.class_binding_is_reassigned
    }

    fn depends_on_class_syntax(self) -> bool {
        self.has_decorators || self.has_auto_accessor
    }
}

/// Number of records kept without allocating.
const INLINE_RECORDS: usize = 8;

/// Stores syntax-dependent [`ClassLowering`] decisions by class scope.
pub struct ClassLoweringDecisions {
    class_transform: ClassTransform,
    decorator_mode: DecoratorMode,
    /// The first few decisions fit inline without allocating.
    entries: SmallVec<[(ScopeId, ClassLowering); INLINE_RECORDS]>,
    spilled: FxHashMap<ScopeId, ClassLowering>,
}

impl ClassLoweringDecisions {
    pub fn new(class_transform: ClassTransform, decorator_mode: DecoratorMode) -> Self {
        Self {
            class_transform,
            decorator_mode,
            entries: SmallVec::new(),
            spilled: FxHashMap::default(),
        }
    }

    /// Record the lowering of a class before its decorators or auto-accessors are rewritten.
    pub fn record(&mut self, class: &Class<'_>) {
        // Legacy decorators run their own lowering, so options alone determine class ownership.
        if self.decorator_mode.is_legacy() {
            // Only class-properties reads `class_binding_is_reassigned`.
            if matches!(self.class_transform, ClassTransform::ClassProperties { .. })
                && legacy_decorators_reassign_class_binding(class)
            {
                let lowering =
                    ClassLowering { class_binding_is_reassigned: true, ..self.default_decision() };
                self.insert(class.scope_id(), lowering);
            }
            return;
        }
        let lowering = ClassLowering::compute(self.class_transform, class);
        if lowering.depends_on_class_syntax() {
            self.insert(class.scope_id(), lowering);
        }
    }

    /// Look up the decision for an entered class, falling back to the program options.
    pub fn get(&self, scope_id: ScopeId) -> ClassLowering {
        self.recorded(scope_id).unwrap_or_else(|| self.default_decision())
    }

    fn recorded(&self, scope_id: ScopeId) -> Option<ClassLowering> {
        if self.spilled.is_empty() {
            self.entries.iter().find(|(id, _)| *id == scope_id).map(|(_, lowering)| *lowering)
        } else {
            self.spilled.get(&scope_id).copied()
        }
    }

    fn insert(&mut self, scope_id: ScopeId, lowering: ClassLowering) {
        if self.spilled.is_empty() {
            if self.entries.len() < INLINE_RECORDS {
                self.entries.push((scope_id, lowering));
                return;
            }
            self.spilled.extend(self.entries.drain(..));
        }
        self.spilled.insert(scope_id, lowering);
    }

    /// The options-based decision for legacy mode and for classes without runtime decorators or
    /// auto-accessors.
    fn default_decision(&self) -> ClassLowering {
        let mode = match self.class_transform {
            ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
            ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
        };
        ClassLowering {
            mode,
            has_decorators: false,
            has_auto_accessor: false,
            class_binding_is_reassigned: false,
        }
    }
}

impl Default for ClassLoweringDecisions {
    fn default() -> Self {
        // Matches the default options: no class lowering, standard decorators preserved.
        Self::new(ClassTransform::default(), DecoratorMode::Preserve)
    }
}

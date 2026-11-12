//! Per-class ownership of class lowering.
//!
//! Several transforms need to know whether class-properties lowers a class, and why. The shape
//! that decision depends on - decorators and auto-accessors - is removed or rewritten early in the
//! transform, so the decision is recorded once per class while the class is still untouched.
//!
//! The first query for a class always happens before any transform mutates it: on entering the
//! class, while capturing the name of a class expression in an initializer, or while scanning a
//! loop body. Later queries read the recorded value.

use oxc_ast::ast::Class;
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    decorator::DecoratorMode,
    utils::decorated_class::{class_body_has_auto_accessors, class_lowered_for_decorators},
};

use super::class_transform::ClassTransform;

/// Which transform owns a class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassLoweringMode {
    /// No transform lowers this class.
    Native,
    /// Class-properties lowers this class because of its decorators or auto-accessors, while the
    /// target supports class fields.
    LowerForDecorators,
    /// Class-properties lowers this class because the target requires class field lowering.
    LowerAll,
    /// The class keeps its decorators and auto-accessors for a downstream transform.
    Preserve,
}

/// Ownership of one class, decided from its untransformed AST.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClassLowering {
    mode: ClassLoweringMode,
    lowered_for_decorators: bool,
    has_auto_accessor: bool,
}

impl ClassLowering {
    fn compute(
        class_transform: ClassTransform,
        decorator_mode: DecoratorMode,
        class: &Class<'_>,
    ) -> Self {
        let lowered_for_decorators = class_lowered_for_decorators(class);
        let has_auto_accessor = class_body_has_auto_accessors(&class.body);
        let mode = if decorator_mode == DecoratorMode::Preserve
            && (lowered_for_decorators || has_auto_accessor)
        {
            ClassLoweringMode::Preserve
        } else {
            match class_transform {
                ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
                ClassTransform::StandardDecorators { .. } => {
                    if lowered_for_decorators {
                        ClassLoweringMode::LowerForDecorators
                    } else {
                        ClassLoweringMode::Native
                    }
                }
                ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
            }
        };
        Self { mode, lowered_for_decorators, has_auto_accessor }
    }

    pub fn mode(self) -> ClassLoweringMode {
        self.mode
    }

    /// Whether the class or one of its elements is decorated.
    pub fn lowered_for_decorators(self) -> bool {
        self.lowered_for_decorators
    }

    pub fn has_auto_accessor(self) -> bool {
        self.has_auto_accessor
    }

    /// Whether standard decorator lowering has anything to do for this class.
    pub fn handles_standard_decorators(self) -> bool {
        self.lowered_for_decorators || self.has_auto_accessor
    }
}

/// Number of records kept without allocating.
const INLINE_RECORDS: usize = 8;

/// Recorded [`ClassLowering`] for every class whose decision is still needed later, keyed by class
/// scope.
pub struct ClassLoweringCache {
    class_transform: ClassTransform,
    decorator_mode: DecoratorMode,
    /// Decorated classes are rare, so the common cases are held inline without allocating.
    entries: SmallVec<[(ScopeId, ClassLowering); INLINE_RECORDS]>,
    spilled: FxHashMap<ScopeId, ClassLowering>,
}

impl ClassLoweringCache {
    pub fn new(class_transform: ClassTransform, decorator_mode: DecoratorMode) -> Self {
        Self {
            class_transform,
            decorator_mode,
            entries: SmallVec::new(),
            spilled: FxHashMap::default(),
        }
    }

    /// Record the lowering of a class which no transform has rewritten yet.
    pub fn get_or_compute(&mut self, class: &Class<'_>) -> ClassLowering {
        let scope_id = class.scope_id();
        if let Some(recorded) = self.recorded(scope_id) {
            debug_assert_eq!(
                recorded,
                ClassLowering::compute(self.class_transform, self.decorator_mode, class),
                "class lowering was queried after the class was rewritten"
            );
            return recorded;
        }
        let lowering = ClassLowering::compute(self.class_transform, self.decorator_mode, class);
        // Only decorators and auto-accessors are removed later. Every other class answers the same
        // after its body is rewritten, so it is not recorded.
        if lowering.handles_standard_decorators() {
            self.record(scope_id, lowering);
        }
        lowering
    }

    /// The lowering of a class whose decision was not recorded.
    pub fn get(&self, scope_id: ScopeId) -> ClassLowering {
        self.recorded(scope_id).unwrap_or_else(|| self.unrecorded())
    }

    fn recorded(&self, scope_id: ScopeId) -> Option<ClassLowering> {
        if self.spilled.is_empty() {
            self.entries.iter().find(|(id, _)| *id == scope_id).map(|(_, lowering)| *lowering)
        } else {
            self.spilled.get(&scope_id).copied()
        }
    }

    fn record(&mut self, scope_id: ScopeId, lowering: ClassLowering) {
        if self.spilled.is_empty() {
            if self.entries.len() < INLINE_RECORDS {
                self.entries.push((scope_id, lowering));
                return;
            }
            self.spilled.extend(self.entries.drain(..));
        }
        self.spilled.insert(scope_id, lowering);
    }

    /// A class without decorators or auto-accessors: only the target decides how it is lowered,
    /// and that does not change when the class body is rewritten.
    fn unrecorded(&self) -> ClassLowering {
        let mode = match self.class_transform {
            ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
            ClassTransform::None
            | ClassTransform::StaticBlocks
            | ClassTransform::StandardDecorators { .. } => ClassLoweringMode::Native,
        };
        ClassLowering { mode, lowered_for_decorators: false, has_auto_accessor: false }
    }
}

impl Default for ClassLoweringCache {
    fn default() -> Self {
        // Matches the default options: no class lowering, standard decorators preserved.
        Self::new(ClassTransform::default(), DecoratorMode::Preserve)
    }
}

//! Per-class ownership of class lowering.
//!
//! Transforms look up the lowering decision by class scope. Decisions that depend on decorators
//! or auto-accessors are recorded at class entry, before any transform can change the class.

use oxc_ast::ast::Class;
use oxc_syntax::scope::ScopeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    decorator::DecoratorMode,
    utils::decorated_class::{class_body_has_auto_accessors, class_has_decorators},
};

use super::class_transform::ClassTransform;

/// Which transform owns a class.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ClassLoweringMode {
    /// Class fields stay native; auto-accessors and static blocks may still be lowered.
    Native,
    /// Class-properties lowers this decorated class even though the target supports class fields.
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
    has_decorators: bool,
    has_auto_accessor: bool,
}

impl ClassLowering {
    fn compute(
        class_transform: ClassTransform,
        decorator_mode: DecoratorMode,
        class: &Class<'_>,
    ) -> Self {
        let has_decorators = class_has_decorators(class);
        let has_auto_accessor = class_body_has_auto_accessors(&class.body);
        let mode = if decorator_mode == DecoratorMode::Preserve
            && (has_decorators || has_auto_accessor)
        {
            ClassLoweringMode::Preserve
        } else {
            match class_transform {
                ClassTransform::ClassProperties { .. } => ClassLoweringMode::LowerAll,
                ClassTransform::StandardDecorators { .. } => {
                    if has_decorators {
                        ClassLoweringMode::LowerForDecorators
                    } else {
                        ClassLoweringMode::Native
                    }
                }
                ClassTransform::None | ClassTransform::StaticBlocks => ClassLoweringMode::Native,
            }
        };
        Self { mode, has_decorators, has_auto_accessor }
    }

    pub fn mode(self) -> ClassLoweringMode {
        self.mode
    }

    /// Whether the class or one of its elements is decorated.
    pub fn has_decorators(self) -> bool {
        self.has_decorators
    }

    pub fn has_auto_accessor(self) -> bool {
        self.has_auto_accessor
    }

    /// Whether standard decorator lowering has anything to do for this class.
    ///
    /// When lowering is enabled, such a class owns per-evaluation state (decorator state or
    /// auto-accessor storage), so each evaluation needs its own block-scoped temporaries. This is
    /// the single definition of that rule; loop scoping, initializer wrapping, and class
    /// temporaries all use it.
    pub fn handles_standard_decorators(self) -> bool {
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
            return;
        }
        let lowering = self.for_untransformed_class(class);
        if lowering.handles_standard_decorators() {
            self.insert(class.scope_id(), lowering);
        }
    }

    /// Inspect a class before its entry hook records its lowering.
    pub fn for_untransformed_class(&self, class: &Class<'_>) -> ClassLowering {
        if self.decorator_mode.is_legacy() {
            return self.default_decision();
        }
        ClassLowering::compute(self.class_transform, self.decorator_mode, class)
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
            ClassTransform::None
            | ClassTransform::StaticBlocks
            | ClassTransform::StandardDecorators { .. } => ClassLoweringMode::Native,
        };
        ClassLowering { mode, has_decorators: false, has_auto_accessor: false }
    }
}

impl Default for ClassLoweringDecisions {
    fn default() -> Self {
        // Matches the default options: no class lowering, standard decorators preserved.
        Self::new(ClassTransform::default(), DecoratorMode::Preserve)
    }
}

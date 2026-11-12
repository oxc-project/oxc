use oxc_ast::ast::Class;

use crate::{
    decorator::DecoratorMode,
    es2022::ClassPropertiesOptions,
    options::EnvOptions,
    utils::decorated_class::{class_body_has_auto_accessors, class_lowered_for_decorators},
};

/// Program-wide ownership of class field and static block lowering.
///
/// Class-properties handles both when field lowering is enabled, so the standalone static block
/// transform must not also run. Otherwise TypeScript handles its field semantics, and static blocks
/// can be lowered independently. TypeScript still removes type syntax and lowers parameter
/// properties in either case.
#[derive(Clone, Copy, Default)]
pub enum ClassTransform {
    #[default]
    None,
    StaticBlocks,
    ClassProperties {
        options: ClassPropertiesOptions,
        transform_static_blocks: bool,
        decorator_mode: DecoratorMode,
    },
    /// Class-properties handles decorated classes and auto-accessors only.
    StandardDecorators {
        transform_static_blocks: bool,
    },
}

impl ClassTransform {
    pub fn new(env: &EnvOptions, decorator_mode: DecoratorMode) -> Self {
        let options = env.es2022;
        if let Some(properties_options) = options.class_properties {
            Self::ClassProperties {
                options: properties_options,
                transform_static_blocks: options.class_static_block,
                decorator_mode,
            }
        } else if decorator_mode == DecoratorMode::Standard {
            Self::StandardDecorators { transform_static_blocks: options.class_static_block }
        } else if options.class_static_block {
            Self::StaticBlocks
        } else {
            Self::None
        }
    }

    /// Whether class-properties owns fields for this class at class entry.
    ///
    /// Callers must retain the result if they need it after decorators and accessors are removed.
    pub fn handles_class_fields(self, class: &Class<'_>) -> bool {
        match self {
            Self::ClassProperties { decorator_mode: DecoratorMode::Preserve, .. } => {
                !class_lowered_for_decorators(class) && !class_body_has_auto_accessors(&class.body)
            }
            Self::ClassProperties { .. } => true,
            Self::StandardDecorators { .. } => class_lowered_for_decorators(class),
            Self::None | Self::StaticBlocks => false,
        }
    }

    /// Whether class-properties owns field lowering and initialization order for all classes.
    ///
    /// TypeScript must leave field lowering to it, and legacy decorators must leave class-alias
    /// initialization to it instead of inserting an alias assignment in a static block.
    pub fn transforms_fields(self) -> bool {
        matches!(self, Self::ClassProperties { decorator_mode, .. } if decorator_mode != DecoratorMode::Preserve)
    }
}

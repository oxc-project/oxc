use crate::es2022::{ClassPropertiesOptions, ES2022Options};

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
    },
}

impl ClassTransform {
    pub fn new(options: ES2022Options) -> Self {
        if let Some(properties_options) = options.class_properties {
            Self::ClassProperties {
                options: properties_options,
                transform_static_blocks: options.class_static_block,
            }
        } else if options.class_static_block {
            Self::StaticBlocks
        } else {
            Self::None
        }
    }

    /// Whether class-properties owns field lowering and initialization order.
    ///
    /// TypeScript must leave field lowering to it, and legacy decorators must leave class-alias
    /// initialization to it instead of inserting an alias assignment in a static block.
    pub fn transforms_fields(self) -> bool {
        matches!(self, Self::ClassProperties { .. })
    }
}

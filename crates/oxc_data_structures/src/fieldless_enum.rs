/// Macro to define a fieldless enum with a `VARIANTS` constant listing all variants in declaration order.
///
/// Wraps the enum definition and adds:
///
/// ```ignore
/// impl EnumName {
///     pub const VARIANTS: [EnumName; N] = [EnumName::A, EnumName::B, ...];
/// }
/// ```
#[macro_export]
macro_rules! fieldless_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(= $discriminant:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $(= $discriminant)?
            ),*
        }

        impl $name {
            /// All variants in declaration order.
            $vis const VARIANTS: [$name; <[&str]>::len(&[$(stringify!($variant)),*])] = [
                $($name::$variant),*
            ];
        }
    };
}

pub use fieldless_enum;

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        fieldless_enum! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            enum Color {
                Red,
                Green,
                Blue,
            }
        }

        assert_eq!(Color::VARIANTS.len(), 3);
        assert_eq!(Color::VARIANTS, [Color::Red, Color::Green, Color::Blue]);
    }

    #[test]
    fn explicit_discriminants() {
        const PENDING: u8 = 10;

        fieldless_enum! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[repr(u8)]
            enum Status {
                Active = 1,
                Inactive = 5,
                Pending = PENDING,
            }
        }

        assert_eq!(Status::VARIANTS.len(), 3);
        assert_eq!(Status::VARIANTS, [Status::Active, Status::Inactive, Status::Pending]);
        assert_eq!(Status::Active as u8, 1);
        assert_eq!(Status::Inactive as u8, 5);
        assert_eq!(Status::Pending as u8, 10);
    }

    #[test]
    fn variant_attributes() {
        fieldless_enum! {
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
            enum WithDefault {
                #[default]
                First,
                Second,
            }
        }

        assert_eq!(WithDefault::default(), WithDefault::First);
        assert_eq!(WithDefault::VARIANTS, [WithDefault::First, WithDefault::Second]);
    }

    #[test]
    fn single_variant() {
        fieldless_enum! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            enum Unit {
                Only,
            }
        }

        assert_eq!(Unit::VARIANTS.len(), 1);
        assert_eq!(Unit::VARIANTS, [Unit::Only]);
    }

    #[test]
    fn zero_variants() {
        fieldless_enum! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            enum Never {}
        }

        assert_eq!(Never::VARIANTS.len(), 0);
        assert_eq!(Never::VARIANTS, []);
    }

    #[test]
    fn declaration_order() {
        fieldless_enum! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            #[repr(u8)]
            enum Shuffled {
                C = 2,
                A = 0,
                B = 1,
            }
        }

        // `VARIANTS` follows declaration order, not discriminant order
        assert_eq!(Shuffled::VARIANTS, [Shuffled::C, Shuffled::A, Shuffled::B]);
    }

    #[test]
    fn visibility() {
        mod inner {
            fieldless_enum! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                pub enum Visible {
                    A,
                    B,
                }
            }
        }

        assert_eq!(inner::Visible::VARIANTS.len(), 2);
    }
}

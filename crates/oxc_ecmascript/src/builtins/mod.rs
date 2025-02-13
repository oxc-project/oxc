use crate::constant_evaluation::ValueType;
use crate::is_global_reference::IsGlobalReference;
use math::MathAbs;
use oxc_ast::ast::{Argument, Expression};
use top_level::{Infinity, NaN, Undefined};
use traits::BuiltinValue;

mod traits;

mod math;
mod top_level;

macro_rules! some_if_exists(
  ($value:literal) => { Some($value) };
  () => { None };
);

macro_rules! declare_builtin {
  ( $( ($path1:literal $(, $path2:literal)? ): $name:ident, )+ ) => {
    pub enum Builtin {
      $($name($name),)+
    }

    impl Builtin {
      pub fn value_type(&self) -> ValueType {
        match self {
          $(Builtin::$name(builtin) => builtin.value_type(),)+
        }
      }

      pub fn may_have_side_effects_on_call(&self, args: &[Argument<'_>]) -> bool {
        match self {
          $(Builtin::$name(builtin) => builtin.may_have_side_effects_on_call(args),)+
        }
      }
    }

    pub trait GetBuiltin: IsGlobalReference {
      fn get_builtin(&self, expr: &Expression) -> Option<Builtin> {
        let (root, property) = match expr {
          Expression::Identifier(ident) => Some((ident, None)),
          Expression::StaticMemberExpression(member_expr) => {
            if let Expression::Identifier(object) = &member_expr.object {
              Some((object, Some(member_expr.property.name.as_str())))
            } else {
              None
            }
          },
          _ => None
        }?;

        if !self.is_global_reference(&root) {
          return None;
        }

        match (root.name.as_str(), property) {
          $(
            ($path1, some_if_exists!($($path2)?)) => Some(Builtin::$name($name)),
          )*
          _ => None,
        }
      }
    }
  };
}

declare_builtin! {
  ("Infinity"): Infinity,
  ("NaN"): NaN,
  ("undefined"): Undefined,
  ("Math", "abs"): MathAbs,
}

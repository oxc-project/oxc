use oxc_ast::ast::*;

use super::super::{MayHaveSideEffects, MayHaveSideEffectsContext};

impl<'a> MayHaveSideEffects<'a> for CallExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if (self.pure && ctx.annotations()) || ctx.manual_pure_functions(&self.callee) {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }

        if let Expression::Identifier(ident) = &self.callee
            && ctx.is_global_reference(ident)
            && let name = ident.name.as_str()
            && (is_pure_global_function(name)
                || is_pure_call(name)
                || is_pure_regexp(name, &self.arguments))
        {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }

        let (object, name) = match &self.callee {
            Expression::StaticMemberExpression(member) if !member.optional => {
                (member.object.get_identifier_reference(), member.property.name.as_str())
            }
            Expression::ComputedMemberExpression(member) if !member.optional => {
                match &member.expression {
                    Expression::StringLiteral(s) => {
                        (member.object.get_identifier_reference(), s.value.as_str())
                    }
                    _ => return true,
                }
            }
            _ => return true,
        };

        let Some(object) = object else { return true };
        if !ctx.is_global_reference(object) {
            return true;
        }

        #[rustfmt::skip]
        let is_global = match object.name.as_str() {
            "Array" => matches!(name, "isArray" | "of"),
            "ArrayBuffer" => name == "isView",
            "Date" => matches!(name, "now" | "parse" | "UTC"),
            "Math" => matches!(name, "abs" | "acos" | "acosh" | "asin" | "asinh" | "atan" | "atan2" | "atanh"
                    | "cbrt" | "ceil" | "clz32" | "cos" | "cosh" | "exp" | "expm1" | "floor" | "fround" | "hypot"
                    | "imul" | "log" | "log10" | "log1p" | "log2" | "max" | "min" | "pow" | "random" | "round"
                    | "sign" | "sin" | "sinh" | "sqrt" | "tan" | "tanh" | "trunc"),
            "Number" => matches!(name, "isFinite" | "isInteger" | "isNaN" | "isSafeInteger" | "parseFloat" | "parseInt"),
            "Object" => matches!(name, "create" | "getOwnPropertyDescriptor" | "getOwnPropertyDescriptors" | "getOwnPropertyNames"
                    | "getOwnPropertySymbols" | "getPrototypeOf" | "hasOwn" | "is" | "isExtensible" | "isFrozen" | "isSealed" | "keys"),
            "String" => matches!(name, "fromCharCode" | "fromCodePoint" | "raw"),
            "Symbol" => matches!(name, "for" | "keyFor"),
            "URL" => name == "canParse",
            "Float32Array" | "Float64Array" | "Int16Array" | "Int32Array" | "Int8Array" | "Uint16Array" | "Uint32Array" | "Uint8Array" | "Uint8ClampedArray" => name == "of",
            _ => false,
        };

        if is_global {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }

        true
    }
}

// `[ValueProperties]: PURE` in <https://github.com/rollup/rollup/blob/master/src/ast/nodes/shared/knownGlobals.ts>
impl<'a> MayHaveSideEffects<'a> for NewExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if (self.pure && ctx.annotations()) || ctx.manual_pure_functions(&self.callee) {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }
        if let Expression::Identifier(ident) = &self.callee
            && ctx.is_global_reference(ident)
            && let name = ident.name.as_str()
            && (is_pure_constructor(name) || is_pure_regexp(name, &self.arguments))
        {
            return self.arguments.iter().any(|e| e.may_have_side_effects(ctx));
        }
        true
    }
}

impl<'a> MayHaveSideEffects<'a> for TaggedTemplateExpression<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        if ctx.manual_pure_functions(&self.tag) {
            self.quasi.may_have_side_effects(ctx)
        } else {
            true
        }
    }
}

impl<'a> MayHaveSideEffects<'a> for Argument<'a> {
    fn may_have_side_effects(&self, ctx: &impl MayHaveSideEffectsContext<'a>) -> bool {
        match self {
            Argument::SpreadElement(e) => match &e.argument {
                Expression::ArrayExpression(arr) => arr.may_have_side_effects(ctx),
                Expression::StringLiteral(_) => false,
                Expression::TemplateLiteral(t) => t.may_have_side_effects(ctx),
                _ => true,
            },
            match_expression!(Argument) => self.to_expression().may_have_side_effects(ctx),
        }
    }
}

fn is_pure_regexp(name: &str, args: &[Argument<'_>]) -> bool {
    name == "RegExp"
        && match args.len() {
            0 | 1 => true,
            2 => args[1].as_expression().is_some_and(|e| {
                matches!(e, Expression::Identifier(_) | Expression::StringLiteral(_))
            }),
            _ => false,
        }
}

#[rustfmt::skip]
pub fn is_pure_global_function(name: &str) -> bool {
    matches!(name, "decodeURI" | "decodeURIComponent" | "encodeURI" | "encodeURIComponent"
            | "escape" | "isFinite" | "isNaN" | "parseFloat" | "parseInt")
}

#[rustfmt::skip]
pub fn is_pure_call(name: &str) -> bool {
    matches!(name, "Date" | "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError"
            | "SyntaxError" | "TypeError" | "URIError" | "Number" | "Object" | "String" | "Symbol")
}

#[rustfmt::skip]
pub fn is_pure_constructor(name: &str) -> bool {
    matches!(name, "Set" | "Map" | "WeakSet" | "WeakMap" | "ArrayBuffer" | "Date"
            | "Boolean" | "Error" | "EvalError" | "RangeError" | "ReferenceError"
            | "SyntaxError" | "TypeError" | "URIError" | "Number" | "Object" | "String" | "Symbol")
}
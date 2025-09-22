use oxc_ast::ast::*;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter},
    generated::ast_nodes::AstNode,
    options::Semicolons,
    write,
};

pub struct OptionalSemicolon;

impl<'a> Format<'a> for OptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        match f.options().semicolons {
            Semicolons::Always => write!(f, ";"),
            Semicolons::AsNeeded => Ok(()),
        }
    }
}

pub struct MaybeOptionalSemicolon(pub bool);

impl<'a> Format<'a> for MaybeOptionalSemicolon {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if self.0 { OptionalSemicolon.fmt(f) } else { Ok(()) }
    }
}

pub struct ClassPropertySemicolon<'a, 'b> {
    element: &'b AstNode<'a, ClassElement<'a>>,
    next_element: Option<&'b AstNode<'a, ClassElement<'a>>>,
}

impl<'a, 'b> ClassPropertySemicolon<'a, 'b> {
    pub fn new(
        element: &'b AstNode<'a, ClassElement<'a>>,
        next_element: Option<&'b AstNode<'a, ClassElement<'a>>>,
    ) -> Self {
        Self { element, next_element }
    }

    fn needs_semicolon(&self) -> bool {
        let Self { element, next_element, .. } = self;

        if let ClassElement::PropertyDefinition(def) = element.as_ref()
            && def.value.is_none()
            && def.type_annotation.is_none()
            && matches!(&def.key, PropertyKey::StaticIdentifier(ident) if matches!(ident.name.as_str(), "static" | "get" | "set") )
        {
            return true;
        }

        let Some(next_element) = next_element else { return false };

        match next_element.as_ref() {
            // When the name starts with the generator token or `[`
            ClassElement::MethodDefinition(def) if !def.value.r#async => {
                (def.computed
                    && !(def.kind.is_accessor()
                        || def.r#static
                        || def.accessibility.is_some()
                        || def.r#override))
                    || def.value.generator
            }
            ClassElement::PropertyDefinition(def) => {
                def.computed
                    && !(def.accessibility.is_some()
                        || def.r#static
                        || def.declare
                        || def.r#override
                        || def.readonly)
            }
            ClassElement::AccessorProperty(def) => {
                def.computed && !(def.accessibility.is_some() || def.r#static || def.r#override)
            }
            ClassElement::TSIndexSignature(_) => true,
            _ => false,
        }
    }
}

impl<'a> Format<'a> for ClassPropertySemicolon<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(
            self.element.as_ref(),
            ClassElement::StaticBlock(_) | ClassElement::MethodDefinition(_)
        ) {
            return Ok(());
        }

        if match f.options().semicolons {
            Semicolons::Always => true,
            Semicolons::AsNeeded => self.needs_semicolon(),
        } {
            write!(f, ";")
        } else {
            Ok(())
        }
    }
}

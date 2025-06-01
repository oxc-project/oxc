use oxc_ast::ast::*;

use crate::{
    formatter::{Buffer, Format, FormatResult, Formatter},
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

pub struct ClassPropertySemicolon<'a, 'b> {
    element: &'b ClassElement<'a>,
    next_element: Option<&'b ClassElement<'a>>,
}

impl<'a, 'b> ClassPropertySemicolon<'a, 'b> {
    pub fn new(element: &'b ClassElement<'a>, next_element: Option<&'b ClassElement<'a>>) -> Self {
        Self { element, next_element }
    }

    fn needs_semicolon(&self) -> bool {
        let Self { element, next_element, .. } = self;

        if let ClassElement::PropertyDefinition(def) = element {
            if def.value.is_none()
                && def.type_annotation.is_none()
                && matches!(&def.key, PropertyKey::StaticIdentifier(ident) if matches!(ident.name.as_str(), "static" | "get" | "set") )
            {
                return true;
            }
        }

        let Some(next_element) = next_element else { return false };
        // TODO
        // `needs_semicolon` in crates/biome_js_formatter/src/js/classes/property_class_member.rs

        match next_element {
            // When the name starts with the generator token or `[`
            ClassElement::MethodDefinition(def) => {
                !def.value.r#async && (def.computed || def.value.generator)
            }
            _ => false,
        }
    }
}

impl<'a> Format<'a> for ClassPropertySemicolon<'a, '_> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        if matches!(
            self.element,
            ClassElement::StaticBlock(_)
                | ClassElement::MethodDefinition(_)
                | ClassElement::TSIndexSignature(_)
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

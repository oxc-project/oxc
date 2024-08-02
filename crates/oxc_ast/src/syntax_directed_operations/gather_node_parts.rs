use oxc_span::Atom;

use crate::ast::*;

// TODO: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L61>
pub trait GatherNodeParts<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F);
}

impl<'a> GatherNodeParts<'a> for Expression<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            Self::Identifier(ident) => f(ident.name.clone()),
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            Self::AssignmentExpression(expr) => expr.left.gather(f),
            Self::UpdateExpression(expr) => expr.argument.gather(f),
            Self::StringLiteral(lit) => lit.gather(f),
            _ => f(Atom::from("ref")),
        }
    }
}

impl<'a> GatherNodeParts<'a> for MemberExpression<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            MemberExpression::ComputedMemberExpression(expr) => {
                expr.object.gather(f);
                expr.expression.gather(f);
            }
            MemberExpression::StaticMemberExpression(expr) => {
                expr.object.gather(f);
                expr.property.gather(f);
            }
            MemberExpression::PrivateFieldExpression(expr) => {
                expr.object.gather(f);
                expr.field.gather(f);
            }
        }
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentTarget<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            match_simple_assignment_target!(Self) => {
                self.to_simple_assignment_target().gather(f);
            }
            match_assignment_target_pattern!(Self) => {}
        }
    }
}

impl<'a> GatherNodeParts<'a> for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            match_member_expression!(Self) => self.to_member_expression().gather(f),
            _ => {}
        }
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierReference<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl<'a> GatherNodeParts<'a> for IdentifierName<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl<'a> GatherNodeParts<'a> for PrivateIdentifier<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl<'a> GatherNodeParts<'a> for StringLiteral<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        f(self.value.clone());
    }
}

use crate::{ast::*, dummy};
use oxc_span::Atom;

// TODO: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L61>
pub trait GatherNodeParts<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F);
}

impl<'a> GatherNodeParts<'a> for Expression<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            Self::Identifier(ident) => f(ident.name.clone()),
            _ if self.is_member_expression() => self.as_member_expression().unwrap().gather(f),
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
            MemberExpression::Dummy => dummy!(),
        }
    }
}

impl<'a> GatherNodeParts<'a> for AssignmentTarget<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            AssignmentTarget::SimpleAssignmentTarget(t) => t.gather(f),
            AssignmentTarget::AssignmentTargetPattern(_) => {}
            AssignmentTarget::Dummy => dummy!(),
        }
    }
}

impl<'a> GatherNodeParts<'a> for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(Atom<'a>)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            _ if self.is_member_expression() => self.as_member_expression().unwrap().gather(f),
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

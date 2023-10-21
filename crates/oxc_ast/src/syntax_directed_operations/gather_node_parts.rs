use crate::ast::*;
use oxc_span::Atom;

// TODO: <https://github.com/babel/babel/blob/419644f27c5c59deb19e71aaabd417a3bc5483ca/packages/babel-traverse/src/scope/index.ts#L61>
pub trait GatherNodeParts {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F);
}

impl<'a> GatherNodeParts for Expression<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        match self {
            Self::Identifier(ident) => f(ident.name.clone()),
            Self::MemberExpression(expr) => expr.gather(f),
            Self::AssignmentExpression(expr) => expr.left.gather(f),
            Self::UpdateExpression(expr) => expr.argument.gather(f),
            Self::StringLiteral(lit) => lit.gather(f),
            _ => f(Atom::from("ref")),
        }
    }
}

impl<'a> GatherNodeParts for MemberExpression<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
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

impl<'a> GatherNodeParts for AssignmentTarget<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        match self {
            AssignmentTarget::SimpleAssignmentTarget(t) => t.gather(f),
            AssignmentTarget::AssignmentTargetPattern(_) => {}
        }
    }
}

impl<'a> GatherNodeParts for SimpleAssignmentTarget<'a> {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        match self {
            Self::AssignmentTargetIdentifier(ident) => ident.gather(f),
            Self::MemberAssignmentTarget(expr) => expr.gather(f),
            _ => {}
        }
    }
}

impl GatherNodeParts for IdentifierReference {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl GatherNodeParts for IdentifierName {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl GatherNodeParts for PrivateIdentifier {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.name.clone());
    }
}

impl GatherNodeParts for StringLiteral {
    fn gather<F: FnMut(Atom)>(&self, f: &mut F) {
        f(self.value.clone());
    }
}

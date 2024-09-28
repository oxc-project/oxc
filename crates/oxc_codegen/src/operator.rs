use oxc_syntax::operator::{BinaryOperator, UnaryOperator, UpdateOperator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Operator {
    Binary(BinaryOperator),
    Unary(UnaryOperator),
    Update(UpdateOperator),
}

impl From<BinaryOperator> for Operator {
    fn from(op: BinaryOperator) -> Self {
        Self::Binary(op)
    }
}

impl From<UnaryOperator> for Operator {
    fn from(op: UnaryOperator) -> Self {
        Self::Unary(op)
    }
}

impl From<UpdateOperator> for Operator {
    fn from(op: UpdateOperator) -> Self {
        Self::Update(op)
    }
}

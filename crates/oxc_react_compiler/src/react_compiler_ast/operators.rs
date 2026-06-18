#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Exp,
    Eq,
    StrictEq,
    Neq,
    StrictNeq,
    Lt,
    Lte,
    Gt,
    Gte,
    Shl,
    Shr,
    UShr,
    BitOr,
    BitXor,
    BitAnd,
    In,
    Instanceof,
    Pipeline,
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    Or,
    And,
    NullishCoalescing,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Plus,
    Not,
    BitNot,
    TypeOf,
    Void,
    Delete,
    Throw,
}

#[derive(Debug, Clone)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

#[derive(Debug, Clone)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    ExpAssign,
    ShlAssign,
    ShrAssign,
    UShrAssign,
    BitOrAssign,
    BitXorAssign,
    BitAndAssign,
    OrAssign,
    AndAssign,
    NullishAssign,
}

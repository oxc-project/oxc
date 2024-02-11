use oxc_syntax::{
    operator::{BinaryOperator, LogicalOperator},
    precedence::{GetPrecedence, Precedence},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BinaryishOperator {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
}

impl From<BinaryOperator> for BinaryishOperator {
    fn from(op: BinaryOperator) -> Self {
        Self::BinaryOperator(op)
    }
}

impl From<LogicalOperator> for BinaryishOperator {
    fn from(op: LogicalOperator) -> Self {
        Self::LogicalOperator(op)
    }
}

impl GetPrecedence for BinaryishOperator {
    fn precedence(&self) -> Precedence {
        match self {
            Self::BinaryOperator(op) => op.precedence(),
            Self::LogicalOperator(op) => op.precedence(),
        }
    }
}

impl BinaryishOperator {
    pub fn should_flatten(self, parent_op: Self) -> bool {
        if self.precedence() != parent_op.precedence() {
            return false;
        }

        let Self::BinaryOperator(op) = self else { return true };

        let Self::BinaryOperator(parent_op) = parent_op else { return true };

        // ** is right-associative
        // x ** y ** z --> x ** (y ** z)
        if parent_op == BinaryOperator::Exponential {
            return false;
        }

        // x == y == z --> (x == y) == z
        if parent_op.is_equality() && op.is_equality() {
            return false;
        }

        // x * y % z --> (x * y) % z
        if (op == BinaryOperator::Remainder && parent_op.is_multiplicative())
            || (parent_op == BinaryOperator::Remainder && op.is_multiplicative())
        {
            return false;
        }

        // x * y / z --> (x * y) / z
        // x / y * z --> (x / y) * z
        if op != parent_op && parent_op.is_multiplicative() && op.is_multiplicative() {
            return false;
        }

        // x << y << z --> (x << y) << z
        if parent_op.is_bitshift() && op.is_bitshift() {
            return false;
        }

        true
    }
}

impl BinaryishOperator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BinaryOperator(op) => op.as_str(),
            Self::LogicalOperator(op) => op.as_str(),
        }
    }
}

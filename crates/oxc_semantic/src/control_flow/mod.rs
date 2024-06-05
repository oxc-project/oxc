mod builder;
mod dot;

use core::fmt;

use oxc_span::CompactStr;
use oxc_syntax::operator::{
    AssignmentOperator, BinaryOperator, LogicalOperator, UnaryOperator, UpdateOperator,
};
use petgraph::{stable_graph::NodeIndex, Graph};

use crate::AstNodeId;

pub use builder::ControlFlowGraphBuilder;

pub type BasicBlockId = NodeIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Index(u32),
    Return,
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyAccessBy {
    PrivateProperty(CompactStr),
    Property(CompactStr),
    Expression(Register),
}

#[derive(Debug, Clone)]
pub struct CollectionAssignmentValue {
    pub id: AstNodeId,
    pub elements: Vec<Register>,
    pub spreads: Vec<usize>,
    pub collection_type: CollectionType,
}

#[derive(Debug, Clone)]
pub struct CalleeWithArgumentsAssignmentValue {
    pub id: AstNodeId,
    pub callee: Register,
    pub arguments: Vec<Register>,
    pub spreads: Vec<usize>,
    pub call_type: CallType,
}

#[derive(Debug, Clone)]
pub struct ObjectPropertyAccessAssignmentValue {
    pub id: AstNodeId,
    pub access_on: Register,
    pub access_by: ObjectPropertyAccessBy,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct BinaryAssignmentValue {
    pub id: AstNodeId,
    pub a: Register,
    pub b: Register,
    pub operator: BinaryOp,
}

#[derive(Debug, Clone)]
pub struct UpdateAssignmentValue {
    pub id: AstNodeId,
    pub expr: Register,
    pub op: UpdateOperator,
    pub prefix: bool,
}

#[derive(Debug, Clone)]
pub struct UnaryExpressioneAssignmentValue(pub AstNodeId, pub UnaryOperator, pub Register);

#[derive(Debug, Clone)]
pub enum AssignmentValue {
    ImplicitUndefined,
    NotImplicitUndefined,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    BinaryOperator(BinaryOperator),
    LogicalOperator(LogicalOperator),
    AssignmentOperator(AssignmentOperator),
}

#[derive(Debug, Clone)]
pub enum CollectionType {
    Array,
    // Note: we do not currently track object names in objects.
    Object,
    JSXElement,
    JSXFragment,
    // doesn't use spreads
    Class,
    TemplateLiteral,
}

#[derive(Debug, Clone)]
pub enum CallType {
    New,
    CallExpression,
    // the callee is the yielded value, arguments are always empty
    // spreads are always empty
    Yield,
    // spreads are always empty
    TaggedTemplate,
    // spreads are always empty
    Import,
}

#[derive(Debug, Clone)]
pub enum BasicBlockElement {
    Unreachable,
    Assignment(Register, AssignmentValue),
    Throw(Register),
    Break(Option<Register>),
}

#[derive(Debug, Clone)]
pub enum EdgeType {
    Normal,
    Backedge,
    NewFunction,
}

#[derive(Debug)]
pub struct ControlFlowGraph {
    pub graph: Graph<usize, EdgeType>,
    pub basic_blocks: Vec<Vec<BasicBlockElement>>,
}

impl ControlFlowGraph {
    /// # Panics
    pub fn basic_block(&self, id: BasicBlockId) -> &Vec<BasicBlockElement> {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get(ix).expect("expected a valid node id in self.basic_blocks")
    }

    /// # Panics
    pub fn basic_block_mut(&mut self, id: BasicBlockId) -> &mut Vec<BasicBlockElement> {
        let ix = *self.graph.node_weight(id).expect("expected a valid node id in self.graph");
        self.basic_blocks.get_mut(ix).expect("expected a valid node id in self.basic_blocks")
    }
}

pub enum StatementControlFlowType {
    DoesNotUseContinue,
    UsesContinue,
}

pub struct PreservedStatementState {
    put_label: bool,
}

pub struct PreservedExpressionState {
    pub use_this_register: Option<Register>,
    pub store_final_assignments_into_this_array: Vec<Vec<Register>>,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index(i) => write!(f, "${i}"),
            Self::Return => write!(f, "$return"),
        }
    }
}

impl fmt::Display for BasicBlockElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Unreachable => f.write_str("Unreachable()"),
            Self::Throw(reg) => write!(f, "throw {reg}"),
            Self::Break(Some(reg)) => write!(f, "break {reg}"),
            Self::Break(None) => f.write_str("break"),
            Self::Assignment(to, with) => {
                // output.push_str(&format!("{} = ", print_register(*to)));
                let value = match with {
                    AssignmentValue::ImplicitUndefined => "<implicit undefined>",
                    AssignmentValue::NotImplicitUndefined => "<value>",
                };
                write!(f, "{to} = {value}")
            }
        }
    }
}

use oxc_syntax::node::NodeId;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
    unreachable: bool,
}

impl BasicBlock {
    pub(crate) fn new() -> Self {
        BasicBlock { instructions: Vec::new(), unreachable: false }
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    #[inline]
    pub fn is_unreachable(&self) -> bool {
        self.unreachable
    }

    #[inline]
    pub fn mark_as_unreachable(&mut self) {
        self.unreachable = true;
    }

    #[inline]
    pub fn mark_as_reachable(&mut self) {
        self.unreachable = false;
    }
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub node_id: Option<NodeId>,
}

impl Instruction {
    pub fn new(kind: InstructionKind, node_id: Option<NodeId>) -> Self {
        Self { kind, node_id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionKind {
    Unreachable,
    Statement,
    Return(ReturnInstructionKind),
    Break(LabeledInstruction),
    Continue(LabeledInstruction),
    Throw,
    Condition,
    Iteration(IterationInstructionKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnInstructionKind {
    ImplicitUndefined,
    NotImplicitUndefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabeledInstruction {
    Labeled,
    Unlabeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IterationInstructionKind {
    Of,
    In,
}

use bitflags::bitflags;
use oxc_syntax::node::NodeId;

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
    flags: BasicBlockFlags,
}

impl BasicBlock {
    pub(crate) fn new(flags: BasicBlockFlags) -> Self {
        BasicBlock { instructions: Vec::new(), flags }
    }

    pub fn flags(&self) -> BasicBlockFlags {
        self.flags
    }

    pub fn instructions(&self) -> &Vec<Instruction> {
        &self.instructions
    }

    #[inline]
    pub fn is_unreachable(&self) -> bool {
        self.flags.contains(BasicBlockFlags::Unreachable)
    }

    #[inline]
    pub fn mark_as_unreachable(&mut self) {
        self.flags.set(BasicBlockFlags::Unreachable, true);
    }

    #[inline]
    pub fn mark_as_reachable(&mut self) {
        self.flags.set(BasicBlockFlags::Unreachable, false);
    }

    #[inline]
    pub(crate) fn set_referenced(&mut self) {
        self.flags |= if self.flags.contains(BasicBlockFlags::Referenced) {
            BasicBlockFlags::Shared
        } else {
            BasicBlockFlags::Referenced
        }
    }
}

bitflags! {
    /// Flags describing a basic block in a [`ControlFlowGraph`](crate::ControlFlowGraph).
    ///
    /// Most of these match TypeScript's `FlowFlags`, but some new flags have
    /// been added for our own use.
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct BasicBlockFlags: u16 {
        // From TypeScript

        /// Unreachable code
        const Unreachable = 1 << 0;
        /// Start of flow graph or subgraph. Could be a program, function,
        /// module, etc.
        const Start = 1 << 1;
        /// Non-looping junction
        const BranchLabel = 1 << 2;
        /// Looping junction
        const LoopLabel = 1 << 3;
        /// Assignment
        const Assignment = 1 << 4;
        /// Condition known to be true
        const TrueCondition = 1 << 5;
        /// Condition known to be false
        const FalseCondition = 1 << 6;
        /// Switch statement clause
        const SwitchClause = 1 << 7;
        /// Potential array mutation
        const ArrayMutation = 1 << 8;
        /// Potential assertion call
        const Call = 1 << 9;
        /// Temporarily reduce antecedents of label
        const ReduceLabel = 1 << 10;
        /// Referenced as antecedent once
        const Referenced = 1 << 11;
        /// Referenced as antecedent more than once
        const Shared = 1 << 12;

        // New flags

        /// A node reached only via implicit control flow
        const Implicit = 1 << 13;
        /// An error harness node
        const Error = 1 << 14;
        const Finalize = 1 << 15;

        const ImplicitError = Self::Implicit.bits() | Self::Error.bits();

        // Also from TypeScript

        const Label = Self::BranchLabel.bits() | Self::LoopLabel.bits();
        const Condition = Self::TrueCondition.bits() | Self::FalseCondition.bits();

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

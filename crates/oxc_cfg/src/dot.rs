// use oxc_ast::{
//     ast::{BreakStatement, ContinueStatement},
//     AstKind,
// };
use petgraph::{
    dot::{Config, Dot},
    visit::EdgeRef,
};

use super::IterationInstructionKind;
use crate::{
    BasicBlock, ControlFlowGraph, EdgeType, Instruction, InstructionKind, LabeledInstruction,
    ReturnInstructionKind,
};

pub trait DisplayDot {
    fn display_dot(&self) -> String;
}

impl DisplayDot for ControlFlowGraph {
    fn display_dot(&self) -> String {
        format!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::EdgeNoLabel, Config::NodeNoLabel],
                &|_graph, edge| {
                    let weight = edge.weight();
                    let label = format!("label = \"{weight:?}\" ");
                    if matches!(weight, EdgeType::Unreachable)
                        || self.basic_block(edge.source()).unreachable
                    {
                        format!("{label}, style = \"dotted\" ")
                    } else {
                        label
                    }
                },
                &|_graph, node| format!(
                    "label = {:?} ",
                    self.basic_blocks[*node.1].display_dot().trim()
                ),
            )
        )
    }
}

impl DisplayDot for BasicBlock {
    fn display_dot(&self) -> String {
        self.instructions().iter().fold(String::new(), |mut acc, it| {
            acc.push_str(it.display_dot().as_str());
            acc.push('\n');
            acc
        })
    }
}

impl DisplayDot for Instruction {
    fn display_dot(&self) -> String {
        match self.kind {
            InstructionKind::Statement => "statement",
            InstructionKind::Unreachable => "unreachable",
            InstructionKind::Throw => "throw",
            InstructionKind::Condition => "condition",
            InstructionKind::Iteration(IterationInstructionKind::Of) => "iteration <of>",
            InstructionKind::Iteration(IterationInstructionKind::In) => "iteration <in>",
            InstructionKind::Break(LabeledInstruction::Labeled) => "break <label>",
            InstructionKind::Break(LabeledInstruction::Unlabeled) => "break",
            InstructionKind::Continue(LabeledInstruction::Labeled) => "continue <label>",
            InstructionKind::Continue(LabeledInstruction::Unlabeled) => "continue",
            InstructionKind::Return(ReturnInstructionKind::ImplicitUndefined) => {
                "return <implicit undefined>"
            }
            InstructionKind::ImplicitReturn => "return",
            InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
                "return <value>"
            }
        }
        .to_string()
    }
}

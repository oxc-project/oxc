use std::{borrow::Cow, fmt};

use itertools::Itertools as _;
use petgraph::{
    dot::{Config, Dot},
    visit::EdgeRef,
};
use rustc_hash::FxHashMap;

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
                    let mut attrs = Attrs::default().with("label", format!("{weight:?}"));

                    if matches!(weight, EdgeType::Unreachable)
                        || self.basic_block(edge.source()).is_unreachable()
                    {
                        attrs += ("style", "dotted");
                    } else if matches!(weight, EdgeType::Error(_)) {
                        attrs += ("color", "red");
                    };

                    format!("{attrs:?}")
                },
                &|_graph, node| {
                    let block = &self.basic_blocks[*node.1];
                    let mut attrs = Attrs::default().with("label", block.display_dot());

                    if *node.1 == 0 {
                        attrs += ("color", "green");
                    }
                    if block.is_unreachable() {
                        attrs += ("style", "dotted");
                    }

                    format!("{attrs:?}")
                },
            )
        )
    }
}

impl DisplayDot for BasicBlock {
    fn display_dot(&self) -> String {
        self.instructions().iter().map(DisplayDot::display_dot).join("\n")
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
            InstructionKind::Return(ReturnInstructionKind::NotImplicitUndefined) => {
                "return <value>"
            }
        }
        .to_string()
    }
}

#[derive(Clone)]
pub enum Attr<'a> {
    String(Cow<'a, str>),
    Identifier(Cow<'a, str>),
    Int(i64),
}
impl<'a> Attr<'a> {
    #[inline]
    #[must_use]
    pub fn ident<S>(identifier: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self::Identifier(identifier.into())
    }
}

impl fmt::Debug for Attr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{i}"),
            Self::String(s) => write!(f, "{s:?}"),
            Self::Identifier(ident) => write!(f, "{ident}"), // display instead of debug
        }
    }
}

impl<'a> From<&'a str> for Attr<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl From<String> for Attr<'static> {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl From<i64> for Attr<'_> {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

#[derive(Default)]
pub struct Attrs<'a>(FxHashMap<Cow<'a, str>, Attr<'a>>);
impl<'a> Attrs<'a> {
    #[must_use]
    #[inline]
    pub fn with<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Attr<'a>>,
    {
        self += (key, value);
        self
    }
}

impl<'a, K, V> FromIterator<(K, V)> for Attrs<'a>
where
    K: Into<Cow<'static, str>>,
    V: Into<Attr<'a>>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

impl<'a, K, V> std::ops::AddAssign<(K, V)> for Attrs<'a>
where
    K: Into<Cow<'static, str>>,
    V: Into<Attr<'a>>,
{
    fn add_assign(&mut self, (key, value): (K, V)) {
        self.0.insert(key.into(), value.into());
    }
}

impl fmt::Debug for Attrs<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }

        let l = self.0.len();
        for (i, (k, v)) in self.0.iter().enumerate() {
            write!(f, "{k}={v:?}")?;
            if i < l - 1 {
                write!(f, ", ")?;
            }
        }

        Ok(())
    }
}

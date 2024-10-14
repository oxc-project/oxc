use super::ControlFlowGraphBuilder;
use crate::{BlockNodeId, EdgeType};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CtxFlags: u8 {
        /// Anything above a `FUNCTION` is unreachable.
        const FUNCTION = 1;
        const BREAK = 1 << 1;
        const CONTINUE = 1 << 2;
    }
}

#[derive(Debug)]
pub(super) struct Ctx<'a> {
    flags: CtxFlags,
    label: Option<&'a str>,
    entries: Vec<(CtxFlags, BlockNodeId)>,
    break_jmp: Option<BlockNodeId>,
    continue_jmp: Option<BlockNodeId>,
}

impl<'a> Ctx<'a> {
    fn new(label: Option<&'a str>, flags: CtxFlags) -> Self {
        Self { flags, label, entries: Vec::new(), break_jmp: None, continue_jmp: None }
    }

    fn is(&self, label: &str) -> bool {
        self.label.as_ref().is_some_and(|it| *it == label)
    }

    fn r#break(&mut self, entry: BlockNodeId) {
        self.entries.push((CtxFlags::BREAK, entry));
    }

    fn r#continue(&mut self, entry: BlockNodeId) {
        self.entries.push((CtxFlags::CONTINUE, entry));
    }
}

pub trait CtxCursor {
    #![allow(clippy::return_self_not_must_use)]
    /// Marks the break jump position in the current context.
    fn mark_break(self, jmp_pos: BlockNodeId) -> Self;
    /// Marks the continue jump position in the current context.
    fn mark_continue(self, jmp_pos: BlockNodeId) -> Self;
    /// Creates a break entry in the current context.
    fn r#break(self, bb: BlockNodeId) -> Self;
    /// Creates a continue entry in the current context.
    fn r#continue(self, bb: BlockNodeId) -> Self;
}

pub struct QueryCtx<'a, 'c>(&'c mut ControlFlowGraphBuilder<'a>, /* label */ Option<&'a str>);

impl<'a, 'c> CtxCursor for QueryCtx<'a, 'c> {
    fn mark_break(self, jmp_pos: BlockNodeId) -> Self {
        self.0.in_break_context(self.1, |ctx| {
            debug_assert!(ctx.break_jmp.is_none());
            ctx.break_jmp = Some(jmp_pos);
        });
        self
    }

    fn mark_continue(self, jmp_pos: BlockNodeId) -> Self {
        self.0.in_continue_context(self.1, |ctx| {
            debug_assert!(ctx.continue_jmp.is_none());
            ctx.continue_jmp = Some(jmp_pos);
        });
        self
    }

    fn r#break(self, bb: BlockNodeId) -> Self {
        self.0.in_break_context(self.1, |ctx| {
            ctx.r#break(bb);
        });
        self
    }

    fn r#continue(self, bb: BlockNodeId) -> Self {
        self.0.in_continue_context(self.1, |ctx| {
            ctx.r#continue(bb);
        });
        self
    }
}

impl<'a, 'c> QueryCtx<'a, 'c> {
    /// Creates a new `Ctx` with the given `CtxFlags` and returns a `RefCtxCursor` to it.
    #[inline]
    #[allow(clippy::wrong_self_convention, clippy::new_ret_no_self)]
    pub fn new(self, flags: CtxFlags) -> RefCtxCursor<'a, 'c> {
        #![allow(unsafe_code)]
        self.0.ctx_stack.push(Ctx::new(self.1, flags));
        // SAFETY: we just pushed this `Ctx` into the stack.
        let ctx = unsafe { self.0.ctx_stack.last_mut().unwrap_unchecked() };
        RefCtxCursor(ctx)
    }

    /// Creates a new `Ctx` with empty `CtxFlags` and returns a `RefCtxCursor` to it.
    pub fn default(self) -> RefCtxCursor<'a, 'c> {
        self.new(CtxFlags::empty())
    }

    /// Creates a new `Ctx` with `CtxFlags::FUNCTION` set as its flags and returns a `RefCtxCursor` to it.
    pub fn new_function(self) -> RefCtxCursor<'a, 'c> {
        self.new(CtxFlags::FUNCTION)
    }

    /// Resolves the current context and adds the required edges to the graph.
    pub fn resolve(mut self) {
        let Some(ctx) = self.0.ctx_stack.pop() else { return };
        self.resolve_ctx(ctx);
    }

    /// Resolves the current context only if the expectations are satisfied.
    ///
    /// # Panics if there is no ctx on the stack or `expectation` isn't satisfied.
    pub fn resolve_expect(mut self, expectation: CtxFlags) {
        let ctx = self.0.ctx_stack.pop().expect("expected a `ctx` on the stack for resolution");
        assert!(ctx.flags.difference(expectation).is_empty());
        self.resolve_ctx(ctx);
    }

    /// Resolves the current context and would mark the possible upper label
    /// continue jump point the same as the resolved context.
    pub fn resolve_with_upper_label(mut self) {
        let Some(ctx) = self.0.ctx_stack.pop() else { return };

        let continue_jmp = ctx.continue_jmp;

        self.resolve_ctx(ctx);

        // mark the upper label continue jump point the same as ours if it isn't already assigned,
        // NOTE: if it is already assigned there's a resolution before this context.
        if let Some(jmp) = continue_jmp {
            if let Some(label_ctx @ RefCtxCursor(Ctx { continue_jmp: None, .. })) =
                self.0.immediate_labeled_ctx()
            {
                label_ctx.mark_continue(jmp);
            }
        }
    }

    /// Resolves the current context and adds the required edges to the graph.
    fn resolve_ctx(&mut self, ctx: Ctx<'a>) {
        // TODO: This match is here to prevent redundant iterations and/or conditions by handling them
        // before starting the iteration, I don't like the current implementation so it would be
        // nice if we find a better way of doing it.
        match (ctx.break_jmp, ctx.continue_jmp) {
            (Some(break_), Some(continue_)) => {
                for entry in ctx.entries {
                    match entry.0 {
                        CtxFlags::BREAK => self.0.add_edge(entry.1, break_, EdgeType::Jump),
                        CtxFlags::CONTINUE => self.0.add_edge(entry.1, continue_, EdgeType::Jump),
                        _ => {}
                    }
                }
            }
            (Some(jmp), None) => {
                for entry in ctx.entries {
                    if matches!(entry.0, CtxFlags::BREAK) {
                        self.0.add_edge(entry.1, jmp, EdgeType::Jump);
                    }
                }
            }
            (None, Some(jmp)) => {
                for entry in ctx.entries {
                    if matches!(entry.0, CtxFlags::CONTINUE) {
                        self.0.add_edge(entry.1, jmp, EdgeType::Jump);
                    }
                }
            }
            (None, None) => {}
        }
    }
}

pub struct RefCtxCursor<'a, 'c>(&'c mut Ctx<'a>);

impl<'a, 'c> RefCtxCursor<'a, 'c> {
    /// Allow break entries in this context.
    pub fn allow_break(self) -> Self {
        self.0.flags.insert(CtxFlags::BREAK);
        self
    }

    /// Allow continue entries in this context.
    pub fn allow_continue(self) -> Self {
        self.0.flags.insert(CtxFlags::CONTINUE);
        self
    }
}

impl<'a, 'c> CtxCursor for RefCtxCursor<'a, 'c> {
    fn mark_break(self, jmp_pos: BlockNodeId) -> Self {
        debug_assert!(self.0.break_jmp.is_none());
        self.0.break_jmp = Some(jmp_pos);
        self
    }

    fn mark_continue(self, jmp_pos: BlockNodeId) -> Self {
        debug_assert!(self.0.continue_jmp.is_none());
        self.0.continue_jmp = Some(jmp_pos);
        self
    }

    fn r#break(self, bb: BlockNodeId) -> Self {
        self.0.r#break(bb);
        self
    }

    fn r#continue(self, bb: BlockNodeId) -> Self {
        self.0.r#continue(bb);
        self
    }
}

impl<'a> ControlFlowGraphBuilder<'a> {
    /// Query a control flow context.
    pub fn ctx<'c>(&'c mut self, label: Option<&'a str>) -> QueryCtx<'a, 'c> {
        QueryCtx(self, label)
    }

    /// Returns `None` if there is no immediate labeled context before this call.
    fn immediate_labeled_ctx<'c>(&'c mut self) -> Option<RefCtxCursor<'a, 'c>> {
        self.ctx_stack.last_mut().filter(|it| it.label.is_some()).map(RefCtxCursor)
    }

    fn in_break_context<F: Fn(&mut Ctx<'a>)>(&mut self, label: Option<&str>, f: F) {
        self.in_context(label, CtxFlags::BREAK, f);
    }

    fn in_continue_context<F: Fn(&mut Ctx<'a>)>(&mut self, label: Option<&str>, f: F) {
        self.in_context(label, CtxFlags::CONTINUE, f);
    }

    fn in_context<F: Fn(&mut Ctx<'a>)>(&mut self, label: Option<&str>, flag: CtxFlags, f: F) {
        let ctx = if let Some(label) = label {
            self.ctx_stack
                .iter_mut()
                .rev()
                // anything up the function is unreachable
                .take_while(|it| !it.flags.intersects(CtxFlags::FUNCTION))
                .filter(|it| it.flags.contains(flag))
                .find(|it| it.is(label))
        } else {
            self.ctx_stack
                .iter_mut()
                .rev()
                // anything up the function is unreachable
                .take_while(|it| !it.flags.intersects(CtxFlags::FUNCTION))
                .find(|it| it.flags.contains(flag))
        };

        if let Some(ctx) = ctx {
            f(ctx);
        }
    }
}

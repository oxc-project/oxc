//! Words: which spellings are keywords here (contextual keywords), the statement keywords,
//! plain names, and member keys with their modifiers.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::*;

#[rustfmt::skip::macros(matches_tk, tk)]
impl Walk {
    /// Step the word at `pos`; returns its end.
    pub(super) fn step_word(&mut self, tokens: &Tokens, pos: usize) -> usize {
        let end = tokens.next_start(pos + 1);
        let kw = if self.after_dot { 0 } else { tokens.word_kw(pos, end - pos) };

        if self.in_type() {
            self.type_word(kw);
            return end;
        }
        // `for await (`: the `await` belongs to the head.
        if kw == tk!(KwAwait) && self.prev_kw == tk!(KwFor) {
            return end;
        }
        let kw = self.resolve_keyword(tokens, end, kw);

        // Member keys in object literals / class bodies.
        if matches!(self.top_kind(), FrameKind::Object | FrameKind::ClassBody)
            && self.top().state == M_KEY_POS
        {
            return self.member_word(tokens, pos, end, kw);
        }
        if self.declared_name(kw) {
            return end;
        }
        self.keyword_word(tokens, end, kw);
        end
    }

    /// A word inside a type: a prefix or infix type keyword, else a type atom.
    fn type_word(&mut self, kw: u8) {
        match kw {
            tk!(
                KwKeyof | KwTypeof | KwReadonly | KwUnique | KwInfer | KwAbstract | KwNew
                | KwAsserts | KwImport | KwExtends | KwIs | KwIn | KwAs | KwSatisfies
            ) => {
                self.type_operator();
                self.prev_kw = kw;
                if kw == tk!(KwExtends) {
                    // A conditional type: the `?` and `:` to come belong to the type, not to an
                    // enclosing expression.
                    if let Some(i) = self.region_index() {
                        self.frames[i].open_questions += 1;
                    }
                }
            }
            _ => {
                // A statement keyword right after a completed type is an error on the same line
                // and was handled by the break rule on a new one; read it as an atom.
                let query = self.prev_kw == tk!(KwTypeof);
                self.type_atom(false);
                // A name takes type arguments, a keyword type only in a type query.
                self.no_type_args = keyword_type(kw) && !query;
            }
        }
    }

    /// Is the spelling with keyword code `kw` a keyword here? A contextual keyword (`yield`,
    /// `await`, `of`, `let`, `using`, `async`, the TypeScript declaration words, `as`,
    /// `satisfies`, `static`, `implements`, `from`) is a plain name (0) unless its position and
    /// the token after it say otherwise; every other code stands.
    fn resolve_keyword(&mut self, tokens: &Tokens, end: usize, kw: u8) -> u8 {
        let keyword = match kw {
            tk!(KwYield | KwAwait) => self.scoped_keyword(kw == tk!(KwYield)),
            tk!(KwOf) => {
                // tsc reads of after any value that ends the first expression of the head.
                self.top_kind() == FrameKind::Head
                    && self.top().state == F_OF
                    && !self.operand_allowed()
            }
            tk!(KwLet) => {
                let nx = tokens.peek(end);
                let binding = nx.kind == tk!(Ident)
                    || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'[' | b'{'));
                binding && (self.at_stmt_start() || self.top_kind() == FrameKind::Head)
            }
            tk!(KwUsing) => {
                let nx = tokens.peek(end);
                nx.kind == tk!(Ident)
                    && !tokens.line_break_between(end, nx.pos)
                    && self.at_stmt_start()
            }
            tk!(KwAsync) => {
                // `async` is a modifier only when the next token is on the same line and continues
                // a function / arrow head.
                let nx = tokens.peek(end);
                (matches_tk!(nx.kind, Ident | String | Number | PrivateIdent)
                    || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'(' | b'*' | b'[')))
                    && !tokens.line_break_between(end, nx.pos)
            }
            tk!(KwType | KwInterface | KwNamespace | KwModule | KwDeclare) => {
                // Statement-level TS declarations only.
                let nx = tokens.peek(end);
                let at_stmt = self.at_stmt_start()
                    || self.prev_kw == tk!(KwDefault)
                    || (kw == tk!(KwNamespace) && self.stmt_reg() == S_EXPORT_AS);
                tokens.ts
                    && matches_tk!(nx.kind, Ident | String)
                    && !tokens.line_break_between(end, nx.pos)
                    && at_stmt
            }
            tk!(KwAs | KwSatisfies) => {
                // Only after a value in an expression, in TS; `export as` opens `export as
                // namespace X`.
                let export_as = kw == tk!(KwAs) && self.prev_kw == tk!(KwExport);
                let in_module_clause = self.top_kind() == FrameKind::ModuleSpec
                    || matches!(self.stmt_reg(), S_IMPORT | S_EXPORT);
                export_as || (tokens.ts && !self.operand_allowed() && !in_module_clause)
            }
            tk!(KwStatic) => {
                self.top_kind() == FrameKind::ClassBody && self.top().state == M_KEY_POS
            }
            tk!(KwImplements) => self.top_kind() == FrameKind::ClassHead,
            tk!(KwFrom) => matches!(self.stmt_reg(), S_IMPORT | S_EXPORT),
            _ => true,
        };
        if keyword { kw } else { 0 }
    }

    /// True when a statement register takes the word as its name (break label, type X, import x).
    fn declared_name(&mut self, kw: u8) -> bool {
        match self.stmt_reg() {
            S_BREAK => {
                // break label, export as namespace N: the statement is complete.
                self.set_stmt_reg(S_NONE);
                self.value_done();
                self.stmt_done = true;
                self.after_statement();
            }
            S_EXPORT_AS if kw == tk!(KwNamespace) => {
                self.set_stmt_reg(S_BREAK);
                self.set_operand();
                self.prev_kw = tk!(KwNamespace);
            }
            _ => return false,
        }
        true
    }

    /// The transition of the keyword `kw` (0: a plain name) in expression or statement position.
    fn keyword_word(&mut self, tokens: &Tokens, end: usize, kw: u8) {
        match kw {
            tk!(KwFunction) => {
                let value = !self.at_stmt_start()
                    && !self.export_default
                    && self.decorator == 0
                    && self.operand_allowed();
                let is_async = self.prev_kw == tk!(KwAsync);
                let f = self.push(FrameKind::FnHead);
                f.is_value = value;
                f.is_async = is_async;
                self.value_done();
                self.prev_kw = tk!(KwFunction);
                self.export_default = false;
            }
            tk!(KwClass) => {
                let value = if self.decorator != 0 {
                    self.decorator == 2
                } else {
                    !self.at_stmt_start() && !self.export_default && self.operand_allowed()
                };
                let f = self.push(FrameKind::ClassHead);
                f.is_value = value;
                self.value_done();
                self.prev_kw = tk!(KwClass);
                self.export_default = false;
                self.decorator = 0;
            }
            tk!(KwIf | KwWhile | KwFor | KwWith | KwSwitch | KwCatch) => {
                self.keyword(kw);
                if kw == tk!(KwCatch) {
                    // `catch {` without a binding.
                    self.expect = Expect::Statement;
                }
            }
            tk!(KwElse | KwDo | KwTry | KwFinally | KwDeclare) => {
                self.expect = Expect::Statement;
                self.clear_prev();
                self.prev_kw = kw;
            }
            tk!(
                KwReturn | KwThrow | KwYield | KwAwait | KwTypeof | KwVoid | KwDelete | KwNew | KwIn
                | KwInstanceof | KwOf | KwDebugger | KwExtends | KwImplements | KwFrom
            ) => {
                if self.top_kind() == FrameKind::Head && matches_tk!(kw, KwOf | KwIn) {
                    self.top_mut().state = F_NO_OF;
                }
                self.keyword(kw);
            }
            tk!(KwCase) => {
                self.set_stmt_reg(S_CASE);
                self.keyword(tk!(KwCase));
            }
            tk!(KwDefault) => {
                let export = self.prev_kw == tk!(KwExport);
                self.export_default |= export;
                self.set_stmt_reg(if export { S_NONE } else { S_CASE });
                self.keyword(kw);
            }
            tk!(KwBreak | KwContinue) => {
                self.set_stmt_reg(S_BREAK);
                self.keyword(kw);
            }
            tk!(KwVar | KwConst | KwLet | KwUsing) => {
                if self.top_kind().is_stmt_holder() {
                    self.top_mut().state = D_BINDING;
                }
                self.keyword(kw);
            }
            tk!(KwImport) => {
                let nx = tokens.peek(end);
                if nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'(' | b'.') {
                    // `import(...)` / `import.meta`: an expression.
                    self.value_done();
                    self.prev_kw = tk!(KwImport);
                } else {
                    self.set_stmt_reg(S_IMPORT);
                    self.keyword(tk!(KwImport));
                }
            }
            tk!(KwExport) => {
                // Only a clause (export {, export *, export type {) keeps the register.
                let mut nx = tokens.peek(end);
                if nx.kind == tk!(Ident) && tokens.ident_is(nx.pos, b"type") {
                    nx = tokens.peek(nx.pos + 4);
                }
                let clause = nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'{' | b'*');
                self.set_stmt_reg(if clause { S_EXPORT } else { S_NONE });
                self.keyword(tk!(KwExport));
                // What follows is read at statement position, as a declaration.
                self.expect = Expect::Statement;
            }
            tk!(KwAs) => {
                if self.prev_kw == tk!(KwExport) {
                    self.set_stmt_reg(S_EXPORT_AS);
                    self.keyword(tk!(KwAs));
                } else {
                    self.open_region(R_EXPR);
                    self.prev_kw = tk!(KwAs);
                }
            }
            tk!(KwSatisfies) => {
                self.open_region(R_EXPR);
                self.prev_kw = tk!(KwSatisfies);
            }
            tk!(KwAsync) => {
                // A modifier: the function or arrow it modifies decides expression-ness, so it is
                // transparent.
                self.clear_prev();
                self.prev_kw = tk!(KwAsync);
            }
            tk!(KwType) => {
                self.set_stmt_reg(S_TYPE_NAME);
                self.keyword(tk!(KwType));
            }
            tk!(KwInterface) => {
                // Same head frame as a class: `extends`, `<` and the body may follow on later
                // lines.
                self.push(FrameKind::ClassHead).reg = C_INTERFACE;
                self.value_done();
                self.prev_kw = tk!(KwInterface);
            }
            tk!(KwModule) if self.prev_kw == tk!(KwDeclare) => {
                // A declared module may have no body.
                self.keyword(kw);
                self.set_stmt_reg(S_DECLARE_MODULE);
            }
            _ => {
                // A plain name, or any other keyword spelling used as one.
                self.plain_word();
            }
        }
    }

    /// A plain identifier (or keyword used as a name) in expression / statement position.
    fn plain_word(&mut self) {
        // `async x => ...`: remember the modifier for the arrow.
        let is_async = self.prev_kw == tk!(KwAsync);
        self.value_done();
        self.arrow_async = is_async;
    }

    /// A word at member-key position of an object literal or class body.
    fn member_word(&mut self, tokens: &Tokens, pos: usize, end: usize, kw: u8) -> usize {
        let is_class = self.top_kind() == FrameKind::ClassBody;
        // Modifiers apply when a key can follow on the same line.
        let nx = tokens.peek(end);
        let key_follows_any_line =
            matches_tk!(nx.kind, Ident | String | Number | BigInt | PrivateIdent)
                || (nx.kind >= OP_KIND_BASE && matches!(nx.byte, b'[' | b'*' | b'#'));
        let key_follows = key_follows_any_line && !tokens.line_break_between(end, nx.pos);
        if kw == tk!(KwAsync) && key_follows {
            self.top_mut().mods |= MOD_ASYNC;
            self.operand_done();
            return end;
        }
        if is_class && kw == tk!(KwStatic) {
            if nx.kind >= OP_KIND_BASE && nx.byte == b'{' {
                self.top_mut().mods |= MOD_STATIC;
                self.keyword(tk!(KwStatic));
                return end;
            }
            if key_follows_any_line {
                self.top_mut().mods |= MOD_STATIC;
                self.operand_done();
                return end;
            }
        }
        if is_class
            && matches_tk!(
                kw,
                KwPublic | KwPrivate | KwProtected | KwReadonly | KwAbstract | KwOverride
                | KwDeclare | KwAccessor
            )
            && key_follows_any_line
        {
            self.operand_done();
            return end;
        }
        if kw == 0
            && (tokens.ident_is(pos, b"get") || tokens.ident_is(pos, b"set"))
            && key_follows_any_line
        {
            self.operand_done();
            return end;
        }
        // The key itself.
        self.top_mut().state = M_KEY_SEEN;
        self.value_done();
        end
    }
}

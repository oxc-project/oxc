//! Punctuation inside a type: type regions and how each kind of region ends, angle lists,
//! and the `<` that may open one.

use crate::{
    cpu::cpu_version,
    token::{OP_KIND_BASE, matches_tk, tk},
};

use super::*;

impl Walk {
    #[cpu_version(Generic)]
    pub(super) fn open_region(&mut self, rule: RegionRule) {
        self.push(FrameKind::TypeRegion).region = rule;
        self.operand_done();
    }

    /// End the type region on top because the token after it belongs to the expression.
    #[cpu_version(Generic)]
    #[inline(always)]
    pub(super) fn end_region_for(&mut self) {
        if self.top_kind() != FrameKind::TypeRegion {
            return;
        }
        let r = self.pop();
        self.set_value();
        match r.region {
            RegionRule::Expr => self.no_type_args = true,
            // The arrow after a return type takes the async of the parameter group.
            RegionRule::ArrowRet => {
                self.closed_group = true;
                self.closed_group_async = r.mods & MOD_ASYNC != 0;
            }
            _ => {}
        }
    }

    /// Can the token at pos, of base kind k, continue the type the region on top completed?
    #[cpu_version(Generic)]
    pub(super) fn continues_type(&self, tokens: &Tokens, pos: usize, k: u8) -> bool {
        let cpu = BASELINE;

        if k == tk!(Ident) {
            // A conditional type or a type predicate.
            return matches_tk!(tokens.ident_kw(pos, cpu), KwExtends | KwIs);
        }
        if k < OP_KIND_BASE {
            return false;
        }
        let (c, c1, r) = (tokens.src[pos], tokens.src[pos + 1], self.top());
        match c {
            b'|' | b'&' => c1 != c,
            b'.' | b'[' => true,
            // Only a name takes type arguments.
            b'<' => !self.no_type_args && c1 != b'=',
            // The rest of a conditional type.
            b'?' | b':' => r.open_questions > 0,
            // The arrow of a function type.
            b'=' => c1 == b'>' && r.inner,
            // An import equals require call.
            b'(' => r.region == RegionRule::Stmt,
            _ => false,
        }
    }

    #[cpu_version(Generic)]
    pub(super) fn type_op(&mut self, pos: usize, c: u8, len: usize) -> usize {
        let top = self.top_kind();
        match c {
            b'(' => {
                self.push(FrameKind::TypeParen);
                self.operand_done();
            }
            b')' => {
                if top == FrameKind::TypeParen {
                    self.pop();
                    self.type_atom(true);
                } else {
                    // Closes something outside the type.
                    self.pop_virtual();
                    self.close_paren();
                }
            }
            b'[' => {
                self.push(FrameKind::TypeBracket);
                self.operand_done();
            }
            b']' => {
                if top == FrameKind::TypeBracket {
                    self.pop();
                    self.type_atom(false);
                } else {
                    self.pop_virtual();
                    self.close_bracket();
                }
            }
            b'{' => {
                self.push(FrameKind::TypeLit);
                self.operand_done();
            }
            b'}' => {
                if top != FrameKind::TypeLit {
                    self.pop_virtual();
                    self.close_brace();
                } else if self.pop().is_interface {
                    // Interface body done: statement over.
                    self.end_statement();
                } else {
                    self.type_atom(false);
                }
            }
            b'<' => {
                let list =
                    if self.operand_allowed() { AngleList::Assert } else { AngleList::InType };
                self.push(FrameKind::Angle).angle = list;
                self.operand_done();
            }
            b'>' if top == FrameKind::Angle => self.close_angle(),
            b';' => {
                if top == FrameKind::TypeLit {
                    self.type_operator();
                } else {
                    self.pop_virtual();
                    self.semicolon();
                }
            }
            b',' | b'=' | b'?' | b':' | b'|' | b'&' | b'.' | b'-' | b'+' | b'*' => {
                if c == b':' && top == FrameKind::TypeRegion && self.top().open_questions > 0 {
                    // The `:` of a conditional type pays its `?`.
                    self.top_mut().open_questions -= 1;
                }
                self.type_operator();
                return pos + len;
            }
            _ => {
                // Any other operator ends an expression-embedded type; in a declaration type it is
                // an error and we treat it the same.
                self.end_region_for();
                self.operand_done();
                return pos + len;
            }
        }
        pos + 1
    }

    #[cpu_version(Generic)]
    pub(super) fn close_angle(&mut self) {
        let a = self.pop();
        match a.angle {
            AngleList::Assert => {
                // Type assertion `<T>`: an operand follows.
                self.type_operator();
            }
            AngleList::Value => {
                // The head or instantiation is a value, and no second list may follow.
                self.value_done();
                self.no_type_args = true;
            }
            AngleList::InType => self.type_atom(false),
        }
    }

    #[cpu_version(Generic)]
    pub(super) fn less_than(&mut self, tokens: &Tokens, pos: usize) {
        let cpu = BASELINE;

        // Declaration type parameters, an assertion or generic arrow, type arguments, or less-than.
        let list = if !tokens.ts {
            None
        } else if self.type_params_expected() {
            Some(AngleList::Value)
        } else if self.operand_allowed() {
            Some(AngleList::Assert)
        } else if !self.no_type_args && type_args_at(tokens, pos, cpu) {
            Some(AngleList::Value)
        } else {
            None
        };
        if let Some(list) = list {
            self.push(FrameKind::Angle).angle = list;
        }
        self.operand_done();
    }
}

//! Punctuation inside a type: type regions and how each kind of region ends, angle lists,
//! and the `<` that may open one.

use crate::token::{OP_KIND_BASE, matches_tk, tk};

use super::*;

impl Walk {
    pub(super) fn open_region(&mut self, rule: u8) {
        self.push(FrameKind::TypeRegion).state = rule;
        self.operand_done();
    }

    /// End the type region on top because the token after it belongs to the expression.
    pub(super) fn end_region_for(&mut self) {
        if self.top_kind() != FrameKind::TypeRegion {
            return;
        }
        let r = self.pop();
        self.set_value();
        match r.state {
            R_EXPR => self.no_type_args = true,
            // The arrow after a return type takes the async of the parameter group.
            R_ARROW_RET => {
                self.closed_group = true;
                self.closed_group_async = r.mods & MOD_ASYNC != 0;
            }
            _ => {}
        }
    }

    /// Can the token at pos, of base kind k, continue the type the region on top completed?
    pub(super) fn continues_type(&self, tokens: &Tokens, pos: usize, k: u8) -> bool {
        if k == tk!(Ident) {
            // A conditional type or a type predicate.
            return matches_tk!(tokens.ident_kw(pos), KwExtends | KwIs);
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
            b'(' => r.state == R_STMT,
            _ => false,
        }
    }

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
                } else if self.pop().state == L_INTERFACE_BODY {
                    // Interface body done: statement over.
                    self.end_statement();
                } else {
                    self.type_atom(false);
                }
            }
            b'<' => {
                let list = if self.operand_allowed() { A_ASSERT } else { A_IN_TYPE };
                self.push(FrameKind::Angle).state = list;
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

    pub(super) fn close_angle(&mut self) {
        let a = self.pop();
        match a.state {
            A_ASSERT => {
                // Type assertion `<T>`: an operand follows.
                self.type_operator();
            }
            A_VALUE => {
                // The head or instantiation is a value, and no second list may follow.
                self.value_done();
                self.no_type_args = true;
            }
            _ => self.type_atom(false),
        }
    }

    pub(super) fn less_than(&mut self, tokens: &Tokens, pos: usize) {
        // Declaration type parameters, an assertion or generic arrow, type arguments, or less-than.
        let list = if !tokens.ts {
            0
        } else if self.type_params_expected() {
            A_VALUE
        } else if self.operand_allowed() {
            A_ASSERT
        } else if !self.no_type_args && type_args_at(tokens, pos) {
            A_VALUE
        } else {
            0
        };
        if list != 0 {
            self.push(FrameKind::Angle).state = list;
        }
        self.operand_done();
    }
}

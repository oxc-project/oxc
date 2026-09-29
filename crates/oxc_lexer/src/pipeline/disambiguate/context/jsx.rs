//! Inside a JSX tag or element only the structure matters: nested tags, containers and the
//! end of the element.

use crate::token::{OP_KIND_BASE, tk};

use super::*;

impl Walk {
    pub(super) fn jsx_element_done(&mut self) {
        // Back in whatever expression held the element (or in a parent's children, where it does
        // not matter).
        if !matches!(self.top_kind(), FrameKind::JsxTag | FrameKind::JsxElem) {
            self.value_done();
        }
    }

    pub(super) fn jsx_lt(&mut self, tokens: &Tokens, pos: usize) -> usize {
        let tpos = tokens.next_sig(pos + 1);
        if tokens.src[tpos] == b'/' {
            // Closing tag: the element it closes is the nearest JsxElem frame.
            if self.pop_to(|k| k == FrameKind::JsxElem).is_none() {
                self.unbalanced();
            }
            self.jsx_closing = true;
        } else {
            self.push(FrameKind::JsxTag);
        }
        pos + 1
    }

    pub(super) fn step_jsx(&mut self, tokens: &Tokens, pos: usize, k: u8) -> usize {
        let c = tokens.src[pos];
        match k {
            tk!(JsxLt) => self.jsx_lt(tokens, pos),
            tk!(JsxTagEnd) => {
                // Self-closing tag or closing tag end.
                self.pop_to(|k| matches!(k, FrameKind::JsxTag | FrameKind::JsxElem));
                self.jsx_element_done();
                pos + 1
            }
            tk!(JsxText | String | Ident | Number | BigInt | TemplateNoSub) => {
                tokens.next_start(pos + 1)
            }
            _ if k >= OP_KIND_BASE => {
                match c {
                    b'{' => {
                        self.push(FrameKind::Container);
                        self.operand_done();
                    }
                    b'<' => {
                        if self.top_kind() == FrameKind::JsxTag {
                            self.top_mut().state += 1;
                        }
                    }
                    b'>' if self.top_kind() == FrameKind::JsxTag => {
                        if self.top().state > 0 {
                            self.top_mut().state -= 1;
                        } else {
                            self.top_mut().kind = FrameKind::JsxElem;
                        }
                    }
                    _ => {}
                }
                pos + 1
            }
            _ => pos + 1,
        }
    }
}

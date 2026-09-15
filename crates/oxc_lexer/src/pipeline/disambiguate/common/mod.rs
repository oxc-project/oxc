//! Building blocks shared by the two disambiguation questions.
//!
//! [`operator`] asks whether a position comes directly after a complete value,
//! which decides whether a `/` is division or starts a regex.
//! [`type_context`] asks whether an angle bracket is part of a TypeScript type.
//! Both answer by walking backwards from the position over tokens which are already lexed,
//! and the parts of those walks which both need live here.
//!
//! There are three layers, each depending only on the ones listed before it:
//!
//! - [`walk`]: moving backwards through the token stream.
//!   Stepping to the previous token, inspecting it, and jumping over bracketed groups.
//! - [`constructs`]: recognizers for individual constructs, e.g. a postfix `!` or the `=` of a type alias.
//!   Each answers one question with a short walk, and none of them recurses.
//! - [`operand`]: whether only an expression can start at a position, and whether a `{` opens a value or a block.
//!   Answering these needs recursion, so this is where functions call each other in a cycle.
//!
//! [`operator`]: super::operator
//! [`type_context`]: super::type_context

use crate::token::TokenKind;

mod constructs;
mod operand;
mod walk;

pub use walk::{bm_prev_sig, memo_new_lex};

pub(super) use constructs::{
    as_gated_type_ref, as_type_operand, bang_is_postfix, class_like_walk,
    conditional_type_question, declarator_without_init, extends_precedes_question,
    incdec_is_postfix, is_binder_keyword, of_is_forof_keyword, return_type_signature_paren,
    signature_return_type, tail_before, type_alias_head, type_prefix_kind,
};
pub(super) use operand::{
    LT_OPERAND_WORDS, annotation_colon_is_declaration, brace_opens_object_literal,
    brace_opens_value, operand_position, type_annotation_asi,
};
pub(super) use walk::{
    AngleMatch, angle_match_back, chain_head, ident_is, kind_at, lt_in_range, match_delim_back,
    prop_name, trivia_at, word_is_any, word_len,
};

pub(super) const KW_ABSTRACT: u8 = TokenKind::KwAbstract as u8;
pub(super) const KW_AS: u8 = TokenKind::KwAs as u8;
pub(super) const KW_ASSERTS: u8 = TokenKind::KwAsserts as u8;
pub(super) const KW_AWAIT: u8 = TokenKind::KwAwait as u8;
pub(super) const KW_BREAK: u8 = TokenKind::KwBreak as u8;
pub(super) const KW_CASE: u8 = TokenKind::KwCase as u8;
pub(super) const KW_CATCH: u8 = TokenKind::KwCatch as u8;
pub(super) const KW_CLASS: u8 = TokenKind::KwClass as u8;
pub(super) const KW_CONST: u8 = TokenKind::KwConst as u8;
pub(super) const KW_CONTINUE: u8 = TokenKind::KwContinue as u8;
pub(super) const KW_DEBUGGER: u8 = TokenKind::KwDebugger as u8;
pub(super) const KW_DEFAULT: u8 = TokenKind::KwDefault as u8;
pub(super) const KW_DELETE: u8 = TokenKind::KwDelete as u8;
pub(super) const KW_DO: u8 = TokenKind::KwDo as u8;
pub(super) const KW_ELSE: u8 = TokenKind::KwElse as u8;
pub(super) const KW_ENUM: u8 = TokenKind::KwEnum as u8;
pub(super) const KW_EXPORT: u8 = TokenKind::KwExport as u8;
pub(super) const KW_EXTENDS: u8 = TokenKind::KwExtends as u8;
pub(super) const KW_FINALLY: u8 = TokenKind::KwFinally as u8;
pub(super) const KW_FOR: u8 = TokenKind::KwFor as u8;
pub(super) const KW_FUNCTION: u8 = TokenKind::KwFunction as u8;
pub(super) const KW_IF: u8 = TokenKind::KwIf as u8;
pub(super) const KW_IMPORT: u8 = TokenKind::KwImport as u8;
pub(super) const KW_IN: u8 = TokenKind::KwIn as u8;
pub(super) const KW_INFER: u8 = TokenKind::KwInfer as u8;
pub(super) const KW_INSTANCEOF: u8 = TokenKind::KwInstanceof as u8;
pub(super) const KW_IS: u8 = TokenKind::KwIs as u8;
pub(super) const KW_KEYOF: u8 = TokenKind::KwKeyof as u8;
pub(super) const KW_LET: u8 = TokenKind::KwLet as u8;
pub(super) const KW_NEW: u8 = TokenKind::KwNew as u8;
pub(super) const KW_READONLY: u8 = TokenKind::KwReadonly as u8;
pub(super) const KW_RETURN: u8 = TokenKind::KwReturn as u8;
pub(super) const KW_SUPER: u8 = TokenKind::KwSuper as u8;
pub(super) const KW_SWITCH: u8 = TokenKind::KwSwitch as u8;
pub(super) const KW_THIS: u8 = TokenKind::KwThis as u8;
pub(super) const KW_THROW: u8 = TokenKind::KwThrow as u8;
pub(super) const KW_TRY: u8 = TokenKind::KwTry as u8;
pub(super) const KW_TYPEOF: u8 = TokenKind::KwTypeof as u8;
pub(super) const KW_UNIQUE: u8 = TokenKind::KwUnique as u8;
pub(super) const KW_VAR: u8 = TokenKind::KwVar as u8;
pub(super) const KW_WHILE: u8 = TokenKind::KwWhile as u8;
pub(super) const KW_WITH: u8 = TokenKind::KwWith as u8;
pub(super) const KW_YIELD: u8 = TokenKind::KwYield as u8;

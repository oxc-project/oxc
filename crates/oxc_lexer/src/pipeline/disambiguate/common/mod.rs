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

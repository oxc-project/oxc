mod fix_conflicts;
mod fix_imports;
mod fix_params;
mod fix_symbol;
mod fix_type_params;
mod fix_vars;

use oxc_syntax::{identifier::is_white_space, line_terminator::is_line_terminator};

use super::{NoUnusedVars, Symbol};
use fix_symbol::BindingInfo;

// source text will never be large enough for this usize to be truncated when
// getting cast to a u32
#[expect(clippy::cast_possible_truncation)]
fn count_whitespace_or_commas<I: Iterator<Item = char>>(iter: I) -> u32 {
    iter.take_while(|&c| c == ',' || is_white_space(c) || is_line_terminator(c))
        .map(|c| c.len_utf8() as u32)
        .sum()
}

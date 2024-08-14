mod fix_symbol;
mod fix_vars;

use super::{NoUnusedVars, Symbol};

use fix_symbol::BindingInfo;

// source text will never be large enough for this usize to be truncated when
// getting cast to a u32
#[allow(clippy::cast_possible_truncation)]
fn count_whitespace_or_commas<I: Iterator<Item = char>>(iter: I) -> u32 {
    iter.take_while(|c| c.is_whitespace() || *c == ',').count() as u32
}

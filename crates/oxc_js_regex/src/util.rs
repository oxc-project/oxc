use phf::phf_set;

const SYNTAX_CHARACTERS: phf::Set<char> = phf_set!['(', ')', '[', ']', '{', '}', '|', '-'];
#[inline]
pub fn is_syntax_character(cp: char) -> bool {
    SYNTAX_CHARACTERS.contains(&cp)
}

use phf::phf_set;

const SYNTAX_CHARACTERS: phf::Set<char> = phf_set!['(', ')', '[', ']', '{', '}', '|', '-'];
#[inline]
pub fn is_syntax_character(cp: char) -> bool {
    SYNTAX_CHARACTERS.contains(&cp)
}

pub fn is_lead_surrogate(code: u32) -> bool {
    code >= 0xd800 && code <= 0xdbff
}

pub fn is_trail_surrogate(code: u32) -> bool {
    code >= 0xdc00 && code <= 0xdfff
}

pub fn combine_surrogate_pair(lead: u32, trail: u32) -> u32 {
    (lead - 0xd800) * 0x400 + (trail - 0xdc00) + 0x10000
}

#[derive(Clone, Copy, Debug)]
pub struct LexOptions {
    pub source_type_module: bool,
    pub jsx: bool,
    pub ts: bool,
    pub validate_regexp_literals: bool,
    pub emit_comments: bool,
    pub emit_whitespace: bool,
    pub collect_line_table: bool,
    pub tolerate_hashbang: bool,
    pub strip_utf8_bom: bool,
    pub validate_utf8: bool,
    pub max_oracle_depth: u32,
    pub max_token_count: u32,
    pub max_diagnostic_count: u32,
}

impl Default for LexOptions {
    fn default() -> Self {
        Self {
            source_type_module: false,
            jsx: false,
            ts: false,
            validate_regexp_literals: false,
            emit_comments: false,
            emit_whitespace: false,
            collect_line_table: false,
            tolerate_hashbang: true,
            strip_utf8_bom: true,
            validate_utf8: false,
            max_oracle_depth: 65536,
            max_token_count: 0,
            max_diagnostic_count: 1024,
        }
    }
}

pub fn default_options() -> LexOptions {
    LexOptions::default()
}

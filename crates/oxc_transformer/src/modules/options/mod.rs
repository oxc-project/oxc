pub mod format;

use format::ModulesFormat;

#[derive(Debug, Default, Clone)]
pub struct ModulesOptions {
    pub format: ModulesFormat,
}

impl ModulesOptions {
    pub fn new(format: ModulesFormat) -> Self {
        Self { format }
    }
}

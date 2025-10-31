pub trait ToolImplementation {
    fn start(&self);
    fn stop(&self);
}

pub struct LintTool;

impl LintTool {
    pub fn new() -> Self {
        LintTool
    }
}

impl ToolImplementation for LintTool {
    fn start(&self) {
        // Start linting tool
    }

    fn stop(&self) {
        // Stop linting tool
    }
}

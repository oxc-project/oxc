use serde_json::Value;

pub trait ExternalFormatterBridge: Send + Sync {
    fn init(&self, num_threads: usize) -> Result<(), String>;
    fn format_file(
        &self,
        options: &Value,
        parser: &str,
        file: &str,
        code: &str,
    ) -> Result<String, String>;
}

#[derive(Debug, Default)]
pub struct NoopBridge;

impl ExternalFormatterBridge for NoopBridge {
    fn init(&self, _num_threads: usize) -> Result<(), String> {
        Ok(())
    }

    fn format_file(
        &self,
        _options: &Value,
        _parser: &str,
        _file: &str,
        _code: &str,
    ) -> Result<String, String> {
        Err("External formatter bridge not configured".to_string())
    }
}

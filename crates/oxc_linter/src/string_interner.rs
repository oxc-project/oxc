use std::collections::HashMap;
use std::sync::LazyLock;

/// A thread-safe string interner for commonly used strings in the linter.
/// This reduces memory allocations by reusing the same string instances.
pub struct StringInterner {
    strings: HashMap<String, &'static str>,
}

impl StringInterner {
    fn new() -> Self {
        Self { strings: HashMap::new() }
    }

    /// Intern a string, returning a static reference to the interned version.
    /// If the string is already interned, returns the existing reference.
    pub fn intern(&mut self, s: String) -> &'static str {
        if let Some(&interned) = self.strings.get(&s) {
            return interned;
        }

        // Leak the string to get a 'static reference
        let leaked: &'static str = Box::leak(s.into_boxed_str());
        self.strings.insert(leaked.to_string(), leaked);
        leaked
    }
}

/// Global string interner instance
static INTERNER: LazyLock<std::sync::Mutex<StringInterner>> =
    LazyLock::new(|| std::sync::Mutex::new(StringInterner::new()));

/// Intern a string globally, returning a static reference.
/// This is thread-safe and can be called from anywhere.
pub fn intern_string(s: String) -> &'static str {
    INTERNER.lock().unwrap().intern(s)
}

/// Pre-computed common URL patterns to avoid repeated formatting
pub struct CommonUrls;

impl CommonUrls {
    /// Generate a rule documentation URL, using string interning for common patterns
    pub fn rule_url(plugin_name: &str, rule_name: &str) -> &'static str {
        let url = format!(
            "https://oxc.rs/docs/guide/usage/linter/rules/{}/{}.html",
            plugin_name, rule_name
        );
        intern_string(url)
    }
}

/// Common string patterns used in diagnostics and formatting
pub struct CommonStrings;

impl CommonStrings {
    /// Get an interned empty string
    pub fn empty() -> &'static str {
        intern_string(String::new())
    }

    /// Get an interned position string like "10:5"
    pub fn position(line: usize, column: usize) -> &'static str {
        intern_string(format!("{}:{}", line, column))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interning() {
        let s1 = intern_string("test".to_string());
        let s2 = intern_string("test".to_string());

        // Should return the same reference
        assert_eq!(s1.as_ptr(), s2.as_ptr());
    }

    #[test]
    fn test_rule_url_generation() {
        let url1 = CommonUrls::rule_url("eslint", "no-unused-vars");
        let url2 = CommonUrls::rule_url("eslint", "no-unused-vars");

        // Should return the same reference for identical URLs
        assert_eq!(url1.as_ptr(), url2.as_ptr());
        assert_eq!(url1, "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-unused-vars.html");
    }
}

/// ESLint suppression handling.
///
/// Port of `Entrypoint/Suppression.ts` from the React Compiler.
///
/// Detects ESLint disable comments that suppress React-related rules,
/// which indicates the code may be breaking React rules and should be
/// skipped during compilation.
/// A suppression range from an eslint-disable comment pair.
#[derive(Debug, Clone)]
pub struct SuppressionRange {
    pub start: u32,
    pub end: Option<u32>,
    pub source: SuppressionSource,
    pub rules: Vec<String>,
}

/// Source of a suppression (ESLint or Flow).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuppressionSource {
    Eslint,
    Flow,
}

/// Default ESLint rules that indicate React rule suppressions.
pub const DEFAULT_ESLINT_SUPPRESSION_RULES: &[&str] =
    &["react-hooks/rules-of-hooks", "react-hooks/exhaustive-deps"];

/// Check if a comment text contains an eslint-disable directive for relevant rules.
pub fn is_eslint_suppression(
    comment_text: &str,
    suppression_rules: &[String],
) -> Option<Vec<String>> {
    let text = comment_text.trim();

    let directive = if let Some(rest) = text.strip_prefix("eslint-disable-next-line") {
        Some(rest)
    } else {
        text.strip_prefix("eslint-disable")
    };

    let directive = directive?;
    let rules_text = directive.trim();
    if rules_text.is_empty() {
        // Blanket disable — matches all rules
        return Some(suppression_rules.to_vec());
    }

    let mut matched_rules = Vec::new();
    for rule in rules_text.split(',') {
        let rule = rule.trim();
        if suppression_rules.iter().any(|r| r == rule) {
            matched_rules.push(rule.to_string());
        }
    }

    if matched_rules.is_empty() { None } else { Some(matched_rules) }
}

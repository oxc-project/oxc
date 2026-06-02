//! Language-agnostic helpers for comment handling.

/// Returns `true` if a comment body is a formatter-suppression marker
/// (`oxfmt-ignore` / `prettier-ignore`). `body` should be the comment text
/// *without* its `//` or `/* */` framing.
pub fn is_suppression_marker(body: &str) -> bool {
    matches!(body.trim(), "oxfmt-ignore" | "prettier-ignore")
}

/// Returns `true` if a comment body is a suppression marker.
/// `body` should be the comment text *without* its `//` or `/* */` framing.
pub fn is_suppression_marker(body: &str) -> bool {
    matches!(body.trim(), "oxfmt-ignore" | "prettier-ignore")
}

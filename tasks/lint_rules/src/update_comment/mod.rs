#[allow(clippy::unnecessary_wraps)] // TODO: Remove this later
pub fn run(_plugin_name: &str, _list: &str) -> Result<String, String> {
    // TODO: Render markdown comment body
    // TODO: Get issue url, comment_id to update by plugin_name
    // TODO: POST to GitHub API(env.GITHUB_TOKEN or PAT may be needed)

    Ok("https://github.com/oxc-project/oxc/issues/...".to_string())
}

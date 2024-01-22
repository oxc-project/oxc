use ureq::Response;

const ESLINT_ISSUE_API_URL: &str = "https://api.github.com/repos/oxc-project/oxc/issues/2117";
// TODO: Restore it after update text is fixed and unsupported list is updated
// const ESLINT_ISSUE_API_URL: &str = "https://api.github.com/repos/oxc-project/oxc/issues/479";

pub fn run(plugin_name: &str, token: &str, comment_body: &str) -> Result<String, String> {
    let api_url = match plugin_name {
        "eslint" => ESLINT_ISSUE_API_URL,
        _ => return Err(format!("😢 Unknown plugin name: {plugin_name}")),
    };

    update_issue_body(api_url, token, comment_body)?;

    Ok(api_url.to_string())
}

fn update_issue_body(url: &str, token: &str, body: &str) -> Result<String, String> {
    let body = oxc_tasks_common::agent()
        .patch(url)
        .set("Accept", "application/vnd.github+json")
        .set("Authorization", &format!("Bearer {token}"))
        .send_json(ureq::json!({ "body": body }))
        .map(Response::into_string);

    match body {
        Ok(Ok(body)) => Ok(body),
        Ok(Err(err)) => Err(err.to_string()),
        Err(err) => Err(err.to_string()),
    }
}

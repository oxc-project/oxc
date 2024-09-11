/**
 * @param {import("./eslint-rules.cjs").TargetPluginMeta} pluginMeta
 * @param {string} markdown
 */
exports.updateGitHubIssue = async ({ issueNo }, markdown) => {
  const issueUrl = `https://github.com/oxc-project/oxc/issues/${issueNo}`;

  try {
    const res = await fetch(
      `https://api.github.com/repos/oxc-project/oxc/issues/${issueNo}`,
      {
        method: 'PATCH',
        headers: {
          Accept: 'application/vnd.github+json',
          Authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
        },
        body: JSON.stringify({ body: markdown }),
      },
    );
    if (!res.ok) throw new Error(res.statusText);
  } catch (err) {
    throw new Error(`Failed to update issue: ${issueUrl}`, { cause: err });
  }

  return `✅ ${issueUrl} is successfully updated`;
};

/**
 * @param {import("./eslint-rules.mjs").TargetPluginMeta} pluginMeta
 * @param {string} markdown
 */
export const updateGitHubIssue = async ({ issueNo }, markdown) => {
  const issueUrl = `https://github.com/oxc-project/oxc/issues/${issueNo}`;

  try {
    const res = await fetch(
      `https://api.github.com/repos/oxc-project/oxc/issues/${issueNo}`,
      {
        Accept: 'application/vnd.github+json',
        Authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
        body: JSON.stringify({ body: markdown }),
        headers: {
          Accept: 'application/vnd.github+json',
          Authorization: `Bearer ${process.env.GITHUB_TOKEN}`,
        },
        method: 'PATCH',
      },
    );
    if (!res.ok) {
      throw new Error(res.statusText);
    }
  } catch (error) {
    throw new Error(`Failed to update issue: ${issueUrl}`, { cause: error });
  }

  return `✅ ${issueUrl} is successfully updated`;
};

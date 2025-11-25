#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Get changed files from GitHub events (pull request or push).
 * This module provides a reusable function for detecting changed files.
 */

const https = require("https");
const process = require("process");

/**
 * Make a GitHub API request
 * @param {string} path - API path
 * @returns {Promise<any>} API response
 */
function githubApi(path) {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: "api.github.com",
      path,
      headers: {
        "User-Agent": "oxc-changed-files",
        Accept: "application/vnd.github.v3+json",
      },
    };

    // Add authorization if token is available
    const token = process.env.GITHUB_TOKEN;
    if (token) {
      options.headers["Authorization"] = `token ${token}`;
    }

    https
      .get(options, (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          if (res.statusCode === 200) {
            resolve(JSON.parse(data));
          } else {
            reject(new Error(`GitHub API error: ${res.statusCode} ${data}`));
          }
        });
      })
      .on("error", reject);
  });
}

/**
 * Get changed files based on the GitHub event type
 * @returns {Promise<string[] | null>} Array of changed file paths, or null to signal "run all"
 */
async function getChangedFiles() {
  const eventName = process.env.GITHUB_EVENT_NAME;
  const repository = process.env.GITHUB_REPOSITORY;
  const sha = process.env.GITHUB_SHA;
  const prNumber = process.env.GITHUB_PR_NUMBER;
  const ref = process.env.GITHUB_REF;

  console.error(`Event: ${eventName}`);
  console.error(`Repository: ${repository}`);
  console.error(`SHA: ${sha}`);
  console.error(`Ref: ${ref}`);

  if (eventName === "workflow_dispatch") {
    console.error("Manual trigger - returning null (run all)");
    return null; // Signal to run all
  }

  let files = [];

  try {
    if (eventName === "pull_request" && prNumber) {
      // For PR, use GitHub API to get changed files
      console.error(`Getting changed files for PR #${prNumber}`);
      const prFiles = await githubApi(`/repos/${repository}/pulls/${prNumber}/files?per_page=100`);
      files = prFiles.map((f) => f.filename);
    } else if (sha && repository) {
      // For push to main, get the commit and compare with parent
      console.error(`Getting changed files for commit ${sha}`);
      const commit = await githubApi(`/repos/${repository}/commits/${sha}`);
      files = commit.files ? commit.files.map((f) => f.filename) : [];
    } else {
      // No valid parameters for API calls
      console.error("Error: Missing required environment variables for GitHub API");
      console.error("Returning null (run all) as fallback");
      return null; // Signal to run all
    }
  } catch (error) {
    console.error(`Error getting changed files via API: ${error.message}`);
    console.error("Returning null (run all) as fallback");
    return null; // Signal to run all
  }

  console.error(`Changed files (${files.length}):`);
  files.forEach((f) => console.error(`  - ${f}`));

  return files;
}

module.exports = { getChangedFiles };

// If run directly as a script, output changed files as JSON
if (require.main === module) {
  getChangedFiles()
    .then((files) => {
      console.log(JSON.stringify(files));
      process.exit(0);
    })
    .catch((error) => {
      console.error("Error:", error);
      console.log(JSON.stringify(null));
      process.exit(1);
    });
}

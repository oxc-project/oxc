#!/usr/bin/env node

// oxlint-disable no-console

/**
 * Get changed files from GitHub events (pull request or push).
 * This module provides a reusable function for detecting changed files.
 */

const fs = require("fs");
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
 * Get all changed files for a PR, paginating through all pages
 * @param {string} repository - Repository name (owner/repo)
 * @param {string} prNumber - Pull request number
 * @returns {Promise<string[]>} Array of changed file paths
 */
async function getPrChangedFiles(repository, prNumber) {
  const perPage = 100;
  const files = [];
  let page = 1;
  let prFiles;
  do {
    // oxlint-disable-next-line no-await-in-loop -- sequential pagination requires await in loop
    prFiles = await githubApi(
      `/repos/${repository}/pulls/${prNumber}/files?per_page=${perPage}&page=${page}`,
    );
    for (const f of prFiles) {
      files.push(f.filename);
    }
    page++;
  } while (prFiles.length >= perPage);
  return files;
}

/**
 * If this PR is in a native GitHub stack, above its lowest unmerged PR, get the commits to compare
 * to find all the stack's changes up to this PR.
 *
 * CI skips most jobs on PRs in the middle of a stack, and relies on the top PR's run to cover their changes
 * (see comment above `jobs` in `.github/workflows/ci.yml`). But a stacked PR's own diff is only against
 * the PR below it, so it'd miss changes made in the PRs below.
 *
 * @returns {{ base: string, head: string } | null} Commits to compare, or `null` if PR's own diff is sufficient
 */
function getStackComparison() {
  const eventPath = process.env.GITHUB_EVENT_PATH;
  if (!eventPath) return null;
  const pr = JSON.parse(fs.readFileSync(eventPath, "utf8")).pull_request;
  const stack = pr?.stack;
  if (!stack || pr.base.ref === stack.base.ref) return null;
  return { base: stack.base.sha || stack.base.ref, head: pr.head.sha };
}

/**
 * Get all files changed between 2 commits (3-dot diff, like a PR's diff)
 * @param {string} repository - Repository name (owner/repo)
 * @param {string} base - Base commit SHA or ref
 * @param {string} head - Head commit SHA
 * @returns {Promise<string[]>} Array of changed file paths
 */
async function getComparisonChangedFiles(repository, base, head) {
  // `compare` API lists at most 300 files. If it hits that limit, the list may be incomplete.
  const MAX_FILES = 300;
  const comparison = await githubApi(`/repos/${repository}/compare/${base}...${head}`);
  if (!Array.isArray(comparison.files) || comparison.files.length >= MAX_FILES) {
    throw new Error(`Comparison of ${base}...${head} may not list all changed files`);
  }
  return comparison.files.map((f) => f.filename);
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
      const stack = getStackComparison();
      if (stack) {
        console.error(
          `PR #${prNumber} is above the bottom of a stack. Getting changed files since stack base ${stack.base}`,
        );
        files = await getComparisonChangedFiles(repository, stack.base, stack.head);
      } else {
        // For PR, use GitHub API to get changed files (paginated)
        console.error(`Getting changed files for PR #${prNumber}`);
        files = await getPrChangedFiles(repository, prNumber);
      }
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

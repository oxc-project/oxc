import { readFileSync } from "node:fs";
import { join } from "node:path";

export interface RepoInfo {
  repoUrl: string;
  commitSha: string;
  version: string;
}

const repos: Record<string, RepoInfo> = JSON.parse(
  readFileSync(join(import.meta.dirname, "../repos.json"), "utf8"),
);

export default repos;

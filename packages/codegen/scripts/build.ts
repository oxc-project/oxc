// Build script.

import { execSync } from "node:child_process";
import { rmSync } from "node:fs";
import { join as pathJoin } from "node:path";

const packageDirPath = pathJoin(import.meta.dirname, "..");

// Delete `dist` directory.
// TSDown's `clean` option can't do it, because both builds output to the same directory,
// so whichever runs second would delete the other's output.
rmSync(pathJoin(packageDirPath, "dist"), { recursive: true, force: true });

// Build both flavours with TSDown
execSync("pnpm tsdown", { stdio: "inherit", cwd: packageDirPath });

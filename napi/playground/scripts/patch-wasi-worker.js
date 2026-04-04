import fs from "node:fs";
import { join as pathJoin } from "node:path";

const path = pathJoin(import.meta.dirname, "../wasi-worker-browser.mjs");

if (fs.existsSync(path)) {
  let data = fs.readFileSync(path, "utf-8");
  data = data.replace(/\nconst errorOutputs = \[\]\n/, "\n");
  fs.writeFileSync(path, data);
}

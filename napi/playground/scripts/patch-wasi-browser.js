import { join as pathJoin } from "node:path";

import { disableReusedWorkers } from "../../disable-reused-workers.mjs";

disableReusedWorkers(pathJoin(import.meta.dirname, "../playground.wasi-browser.js"));

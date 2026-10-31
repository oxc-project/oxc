// Preloaded into the CLI process and every Prettier worker through `NODE_OPTIONS`.
// Only a worker has an IPC channel (`process.send`).
// Each worker records its PID, then leaves a referenced timer,
// like a Prettier plugin that never clears one.
import { appendFileSync } from "node:fs";

if (process.send !== undefined) {
  appendFileSync(process.env.OXFMT_TEST_WORKER_PID_FILE, `${process.pid}\n`);
  setInterval(() => {}, 1000);
}

// Simulate a plugin that keeps a worker alive. Only workers have `process.send`.
import { appendFileSync } from "node:fs";

if (process.send !== undefined) {
  appendFileSync(process.env.OXFMT_TEST_WORKER_PID_FILE, `${process.pid}\n`);
  setInterval(() => {}, 1000);
}

// Leaves async work behind after the config is loaded, like a Vite plugin or a telemetry hook.
// The CLI must still run all of it before it exits, as a natural Node.js exit does.
import { createWriteStream } from "node:fs";

const out = process.env.OXFMT_TEST_OUT_FILE;
if (out) {
  // `beforeExit` fires after the formatting is done. The work it adds must finish too,
  // and the exit code that it sets must win.
  let started = false;
  process.on("beforeExit", () => {
    if (started) return;
    started = true;
    setTimeout(() => {
      // An unawaited stream write that needs many event-loop turns to finish.
      const stream = createWriteStream(out);
      const chunk = Buffer.alloc(65_536, 97);
      for (let i = 0; i < 64; i++) stream.write(chunk);
      stream.end(() => {
        process.exitCode = 7;
      });
    }, 20);
  });
}

export default {};

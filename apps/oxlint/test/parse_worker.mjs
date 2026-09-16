// Worker for `parse.test.ts`: parses, then hands the raw transfer buffer to the parent thread
// via `ArrayBuffer.prototype.transfer` (which launders the external buffer into a transferable one),
// then exits. The parent must still be able to read it after this thread is gone.
import { parentPort } from "node:worker_threads";
import { getRawTransferBuffer, parseRawSync } from "../src-js/bindings.js";

parseRawSync("dummy.js", "let a = 1;");
const view = getRawTransferBuffer();
view[4096] = 0x5a;
const laundered = view.buffer.transfer();
parentPort.postMessage(laundered, [laundered]);

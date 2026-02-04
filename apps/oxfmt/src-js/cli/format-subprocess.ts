/**
 * Formatting subprocess for Node.js v24 workaround.
 *
 * This script runs in a child process and handles Prettier formatting requests.
 * By running in a separate process (not worker_threads), we completely avoid
 * the ThreadsafeFunction race condition bug in Node.js v24.
 *
 * Communication is via Node.js IPC channel (process.send/on('message')).
 *
 * See: https://github.com/nodejs/node/issues/55706
 */
import { formatEmbeddedCode, formatFile, sortTailwindClasses } from "../libs/prettier";

interface Request {
  id: number;
  method: "formatEmbeddedCode" | "formatFile" | "sortTailwindClasses";
  args: unknown;
}

interface Response {
  id: number;
  result?: unknown;
  error?: string;
}

process.on("message", (request: Request) => {
  void handleMessage(request);
});

async function handleMessage(request: Request): Promise<void> {
  const response: Response = { id: request.id };

  try {
    switch (request.method) {
      case "formatEmbeddedCode":
        response.result = await formatEmbeddedCode(
          request.args as Parameters<typeof formatEmbeddedCode>[0],
        );
        break;
      case "formatFile":
        response.result = await formatFile(request.args as Parameters<typeof formatFile>[0]);
        break;
      case "sortTailwindClasses":
        response.result = await sortTailwindClasses(
          request.args as Parameters<typeof sortTailwindClasses>[0],
        );
        break;
      default:
        response.error = `Unknown method: ${request.method}`;
    }
  } catch (e) {
    response.error = e instanceof Error ? e.message : String(e);
  }

  process.send!(response);
}

// Signal ready
process.send!({ ready: true });

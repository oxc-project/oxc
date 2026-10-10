import fs from "node:fs";

const REUSED_WORKERS = "reuseWorker: { size: __asyncWorkPoolSize + __workerPoolSize },";
const DISABLED_REUSED_WORKERS = "reuseWorker: false,";

export function disableReusedWorkers(path) {
  let data = fs.readFileSync(path, "utf-8");

  // The pool is initialized eagerly, but browsers reject its worker URL when a binding is loaded
  // from a cross-origin CDN because workers must be loaded from the page's origin.
  if (data.includes(REUSED_WORKERS)) {
    data = data.replace(REUSED_WORKERS, DISABLED_REUSED_WORKERS);
    fs.writeFileSync(path, data);
  } else if (!data.includes(DISABLED_REUSED_WORKERS)) {
    throw new Error(`Could not find the reuseWorker option in ${path}`);
  }
}

/**
 * Node.js 24.2.0 - 24.21.x, 25.x and 26.0 - 26.9 have a use-after-free when a `Worker` is freed:
 * a cppgc wrapper (e.g. a `vm.Script` from module loading) that is collected shortly before
 * its Environment is freed, but swept afterwards, writes into the freed Realm.
 * It was introduced by nodejs/node#56534 and fixed by nodejs/node#65778 (24.22.0, 26.10.0).
 *
 * Destroying the pool at the end of a run frees every worker at once,
 * and on Windows this crashes with `STATUS_ACCESS_VIOLATION` in a good share of runs.
 * (https://github.com/oxc-project/oxc/issues/29386)
 *
 * On those versions, run the pool in `child_process` workers instead,
 * where each worker is a separate process and no Environment is freed in this process.
 */
export function hasWorkerTeardownBug(nodeVersion: string): boolean {
  const [major, minor] = nodeVersion.split(".").map(Number);
  if (major === 24) return minor >= 2 && minor < 22;
  if (major === 25) return true;
  if (major === 26) return minor < 10;
  return false;
}

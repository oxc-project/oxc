// @outputMode:"lint"
function Component() {
  try {
    doWork();
  } catch {
    recover();
  } finally {
    cleanup();
  }
}

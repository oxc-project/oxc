export function useGuard(save) {
  return { run: save, markSaved() {} };
}

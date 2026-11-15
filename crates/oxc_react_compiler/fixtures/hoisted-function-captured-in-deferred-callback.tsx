import {useGuard} from './guard.mjs';

export function View() {
  const guard = useGuard(() => save());
  async function save() {
    await Promise.resolve();
    guard.markSaved();
    return true;
  }
  return <button onClick={save}>Save</button>;
}

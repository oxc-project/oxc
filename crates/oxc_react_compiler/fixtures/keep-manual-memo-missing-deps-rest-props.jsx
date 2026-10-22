// @compilationMode:"infer" @panicThreshold:"none"
// https://github.com/oxc-project/oxc/issues/27113
// The dependency list is incomplete on purpose; the useMemo is kept as written.
import { useMemo } from 'react';

// `values` is recomputed only when `items` changes, on purpose:
// `s` / `props.value` are left out so the list does not re-sort while the user interacts with it.
export function List({ s, ...props }) {
  const { items, children } = props;
  const values = useMemo(() => sortBy(items, s ?? props.value), [items]);
  return children(values);
}

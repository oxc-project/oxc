// @compilationMode:"infer" @panicThreshold:"none"
// React calls the callback only when `user.id` changes, so `user.profile.items` must not be
// read on every render: it may be null on a render that reuses the memoized value.
import { useMemo } from 'react';

function Profile({ user, s }) {
  const items = useMemo(() => sortBy(user.profile.items, s), [user.id]);
  return <span>{items}</span>;
}

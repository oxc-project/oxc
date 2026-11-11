import { useEffect } from 'react';
export function Component() {
  useEffect(() => {
    let config = { onEvent: () => {} };
    subscribe(() => {
      config = { ...config };
    });
  }, []);
  return <div />;
}

import React, { useEffect } from 'react';

function Component() {
  const emit = (event) => console.log(event);
  const EVENTS = { CLEAR: 'clear' };

  useEffect(() => {
    emit(EVENTS.CLEAR);
    // oxlint-disable-next-line exhaustive-deps
  }, []);

  return null;
}


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

// Test case for issue #14233 - disable directive not working for rules-of-hooks
function useMostRelevantBreakdownType(params, filters) {
  // Helper function that starts with "use" but isn't a React hook
  console.log(params, filters);
}

const cleanBreakdownParams = (cleanedParams, filters) => {
  // this isn't a react hook
  // oxlint-disable-next-line react-hooks/rules-of-hooks
  useMostRelevantBreakdownType(cleanedParams, filters);
}


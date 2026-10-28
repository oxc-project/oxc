export interface RuleTiming {
  ruleIndex: number;
  durationMs: number;
  calls: number;
}

export function timeCall<ReturnValue>(
  timing: RuleTiming,
  callback: () => ReturnValue,
): ReturnValue {
  const start = performance.now();
  try {
    return callback();
  } finally {
    timing.durationMs += performance.now() - start;
    timing.calls++;
  }
}

export function wrapTimedFunction<Args extends unknown[], ReturnValue>(
  fn: (...args: Args) => ReturnValue,
  timing: RuleTiming,
): (...args: Args) => ReturnValue {
  return (...args) => {
    const start = performance.now();
    try {
      return fn(...args);
    } finally {
      timing.durationMs += performance.now() - start;
      timing.calls++;
    }
  };
}

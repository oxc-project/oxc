// Examples of incorrect code for use-unknown-in-catch-callback-variable rule

// Should use 'unknown' instead of 'any'
try {
  riskyOperation();
} catch (error: any) {
  // Should be: error: unknown
  console.log(error.message);
}

// Implicit 'any' in catch clause
try {
  riskyOperation();
} catch (error) {
  // Implicitly 'any', should be: error: unknown
  handleError(error);
}

// Promise catch with explicit any
promise.catch((error: any) => {
  // Should be: error: unknown
  console.error(error.message);
});

// Callback with any error type
function handleAsync(callback: (error: any) => void) {
  // Should be: error: unknown
  try {
    performOperation();
    callback(null);
  } catch (err) {
    callback(err);
  }
}

declare function riskyOperation(): void;
declare function handleError(error: unknown): void;
declare const promise: Promise<any>;
declare function performOperation(): void;

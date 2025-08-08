// Examples of incorrect code for use-unknown-in-catch-callback-variable rule

try {
  somethingRisky();
} catch (error: any) { // Should use 'unknown'
  console.log(error.message); // Unsafe access
  error.someMethod(); // Unsafe call
}

// Default catch variable is 'any' in older TypeScript
try {
  somethingRisky();
} catch (error) { // Should explicitly type as 'unknown'
  throw error;
}
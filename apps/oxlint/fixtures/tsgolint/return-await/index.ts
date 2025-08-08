// Examples of incorrect code for return-await rule
// This rule requires consistent return await usage

async function fetchUser(): Promise<User> {
  // Should be: return await getUser();
  return getUser(); // Missing await
}

async function processData(): Promise<string> {
  try {
    // Should be: return await fetchData();
    return fetchData(); // Missing await in try block
  } catch (error) {
    throw new Error('Failed to process');
  }
}

// In try-catch, return await is recommended for proper stack traces
async function handleRequest(): Promise<Response> {
  try {
    return handleAsync(); // Should use await
  } catch (error) {
    console.error(error);
    throw error;
  }
}

declare function getUser(): Promise<User>;
declare function fetchData(): Promise<string>;
declare function handleAsync(): Promise<Response>;
declare interface User { id: number; }
declare interface Response { data: any; }
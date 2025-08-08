// Examples of incorrect code for return-await rule

// If configured to require await:
async function fetchData() {
  return fetch('/api/data'); // Should be: return await fetch('/api/data');
}

async function processData() {
  return someAsyncOperation(); // Should be: return await someAsyncOperation();
}
// Examples of incorrect code for require-await rule

// Async function without await
async function fetchData() {
  return fetch('/api/data');
}

// Async arrow function without await
const processData = async () => {
  return someData.map(x => x * 2);
};
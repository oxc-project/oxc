// Examples of incorrect code for promise-function-async rule

// Function returning Promise without async
function fetchData(): Promise<string> {
  return fetch('/api/data').then(res => res.text());
}

// Method returning Promise without async
class DataService {
  getData(): Promise<any> {
    return fetch('/api/data').then(res => res.json());
  }
}
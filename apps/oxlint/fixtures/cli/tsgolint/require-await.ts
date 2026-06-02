declare function fetch(url: string): Promise<Response>;
async function fetchData() {
  return fetch('/api/data');
}
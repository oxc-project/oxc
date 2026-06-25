declare function fetch(url: string): Promise<Response>;
function fetchData(): Promise<string> {
  return fetch('/api/data').then(res => res.text());
}

export {};

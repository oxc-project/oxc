// Examples of incorrect code for prefer-string-starts-ends-with rule

function startsCase(s: string): boolean {
  return s[0] === 'a';
}

function endsCase(s: string): boolean {
  return s.slice(-3) === 'bar';
}

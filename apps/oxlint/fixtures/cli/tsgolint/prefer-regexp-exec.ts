// Examples of incorrect code for prefer-regexp-exec rule

const text = 'something';
text.match(/thing/); // should prefer RegExp#exec

const search = 'thing';
text.match(search); // should prefer RegExp#exec

function countMatches(str: string, re: RegExp): number {
  return str.match(re)?.length ?? 0; // valid: unknown global flag
}

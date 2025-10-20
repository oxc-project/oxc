// Examples of incorrect code for no-unsafe-return rule

declare const anyValue: any;

function getString(): string {
  return anyValue; // unsafe return
}

const getNumber = (): number => anyValue; // unsafe return

function processData(): { name: string; age: number } {
  return anyValue; // unsafe return
}

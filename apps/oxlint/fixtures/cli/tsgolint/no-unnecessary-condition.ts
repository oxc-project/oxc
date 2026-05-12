declare const b1: object;
declare const b2: boolean;
export const t1 = b1 && b2;

function checkNumber(x: number) {
  // eslint-disable-next-line typescript/no-unnecessary-condition
  if (x != null) {
    return x;
  }
  return 0;
}

checkNumber(1);

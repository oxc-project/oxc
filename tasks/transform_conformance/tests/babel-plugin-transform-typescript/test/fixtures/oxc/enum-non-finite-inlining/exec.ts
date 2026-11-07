const infinity = Infinity;
const nan = NaN;
enum Values { Positive = infinity, Negative = -infinity, NotANumber = nan }
function nonFinite(Infinity: number, NaN: number) {
  return [Values.Positive, Values.Negative, Values.NotANumber];
}
const values = nonFinite(7, 9);
expect(values[0]).toBe(Infinity);
expect(values[1]).toBe(-Infinity);
expect(values[2]).toBeNaN();

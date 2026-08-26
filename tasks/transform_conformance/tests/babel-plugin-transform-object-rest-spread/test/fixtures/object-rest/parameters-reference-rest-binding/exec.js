function readAfter({ value, ...rest }, selected = value) {
  return [rest, selected];
}

expect(readAfter({ value: 1, other: 2 })).toEqual([{ other: 2 }, 1]);
expect(readAfter({ value: 1, other: 2 }, 3)).toEqual([{ other: 2 }, 3]);

const rest = "outer";
function readBefore(selected = rest, { ...rest }) {
  return selected;
}

expect(() => readBefore(undefined, { other: 2 })).toThrow(ReferenceError);

const value = "outer";
const receiver = {
  readShadowed({ ...rest }, selected = rest, shadowed = value) {
    let value;
    return [this, arguments[0], rest, selected, shadowed];
  },
};

const argument = { other: 2 };
expect(receiver.readShadowed(argument)).toEqual([
  receiver,
  argument,
  { other: 2 },
  { other: 2 },
  "outer",
]);

expect(readAfter.length).toBe(1);
expect(readBefore.length).toBe(0);

it.only("does something really long and descriptive with the formatter output, over the print width", async () => {
  expect(1).toBe(1);
});

it.concurrent("does something really long and descriptive with the formatter output, over the print width", async () => {
  expect(1).toBe(1);
});

describe.shuffle("does something really long and descriptive with the formatter output, over the print width", () => {
  run();
});

test.fails("does something really long and descriptive with the formatter output, over the print width", () => {
  run();
});

bench("does something really long and descriptive with the formatter output, over the print width", () => {
  run();
});

Deno.test("does something really long and descriptive with the formatter output, over the print width", () => {
  run();
});

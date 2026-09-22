test("timeout", () => {
  run();
},
// first
// second
// third
15000);

it("callback", // first
// second
function (done) {
  done();
});

describe.only(
// description
"suite", () => {
  run();
});

test("trailing", () => {
  run();
}, 15000 // first
// second
);

test("inside callback", () => {
  // body
  run();
});

const nestedAwait = async () =>
  void (await consume(async () =>
    (await import("a-very-long-module-name-that-forces-the-expression-to-break")).default(),
  ));

const assignmentChain = () =>
  firstVeryLongIdentifier = secondVeryLongIdentifier = thirdVeryLongIdentifier;

const amd = () =>
  define("a-very-long-module-name", ["first-long-dependency", "second-long-dependency"], () => {
    return value;
  });

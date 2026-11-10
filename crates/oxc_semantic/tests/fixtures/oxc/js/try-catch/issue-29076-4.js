try {
  throw "o";
} catch (x) {
  try {
    throw "i";
  } catch (x) {
    const read = () => x;
    {
      var { value: x } = { value: "v" };
    }
    console.log(read(), x);
  }
  console.log(x);
}

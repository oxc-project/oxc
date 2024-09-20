function outer() {
  let f = () => {
    {
      let t = this;
    }
  };

  let f2 = () => {
    if (x) {
      if (y) {
        return this;
      }
    }
  };
}

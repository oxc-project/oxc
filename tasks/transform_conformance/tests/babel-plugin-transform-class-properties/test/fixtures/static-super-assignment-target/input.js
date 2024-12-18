let bound = "A";

class Outer {
  static prop = () => {
    [super.prop] = [];
    [super[bound]] = [];
    [super[unbound]] = [];

    [...super.prop] = [];
    [...super[bound]] = [];
    [...super[unbound]] = [];

    ({x: super.prop} = {});
    ({x: super[bound]} = {});
    ({x: super[unbound]} = {});

    ({...super.prop} = {});
    ({...super[bound]} = {});
    ({...super[unbound]} = {});
  };
}

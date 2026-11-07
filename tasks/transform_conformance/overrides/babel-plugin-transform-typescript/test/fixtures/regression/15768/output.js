const v = 42;
const v2 = Infinity;
const v3 = NaN;
{
  let Infinity = 1;
  let NaN = 2;
  let StateEnum = /* @__PURE__ */ function(StateEnum) {
    StateEnum[StateEnum["okay"] = 0] = "okay";
    StateEnum[StateEnum["neg"] = -Infinity] = "neg";
    StateEnum[StateEnum["pos"] = Infinity] = "pos";
    StateEnum[StateEnum["nan"] = NaN] = "nan";
    StateEnum[StateEnum["negReal"] = -(1 / 0)] = "negReal";
    StateEnum[StateEnum["posReal"] = 1 / 0] = "posReal";
    StateEnum[StateEnum["nanReal"] = 0 / 0] = "nanReal";
    StateEnum[StateEnum["ext"] = 42] = "ext";
    StateEnum[StateEnum["ext2"] = 1 / 0] = "ext2";
    StateEnum[StateEnum["ext3"] = 0 / 0] = "ext3";
    return StateEnum;
  }({});
}

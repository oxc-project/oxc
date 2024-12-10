let boundProp, mutatedProp;
mutatedProp = "x";

class C extends S {
  method() {
    var _unboundProp, _mutatedProp;

    super[boundProp] && (super[boundProp] = 1);
    super[(_unboundProp = unboundProp)] && (super[_unboundProp] = 2);
    super[(_mutatedProp = mutatedProp)] && (super[_mutatedProp] = 3);
  }
}

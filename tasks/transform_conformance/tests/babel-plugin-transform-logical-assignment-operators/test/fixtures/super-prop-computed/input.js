let boundProp, mutatedProp;
mutatedProp = 'x';

class C extends S {
  method() {
    super[boundProp] &&= 1;
    super[unboundProp] &&= 2;
    super[mutatedProp] &&= 3;
  }
}

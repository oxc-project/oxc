@dec
class C {
  prop = 0;
  meth() {
    return this.prop;
  }
}

@dec
export class D {
  prop = 0;
  meth() {
    return this.prop;
  }
}

@dec
export default class E {
  prop = 0;
  meth() {
    return this.prop;
  }
}

class F {
  @dec
  prop = 0;
}

export class G {
  @dec
  prop = 0;
}
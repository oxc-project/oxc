class C {
  #p = 1;

  method(obj) {
    this.#p **= 1;
    obj.#p **= 2;
    this.x.y.z.#p **= 3;
    obj.x.y.z.#p **= 4;
    fn().x.y.z.#p **= 5;
  }
}

class C {
  #p = 1;

  method(obj) {
    var _this, _this$x$y$z, _obj$x$y$z, _fn$x$y$z;
    _this = this, _this.#p = Math.pow(_this.#p, 1);
    obj.#p = Math.pow(obj.#p, 2);
    _this$x$y$z = this.x.y.z, _this$x$y$z.#p = Math.pow(_this$x$y$z.#p, 3);
    _obj$x$y$z = obj.x.y.z, _obj$x$y$z.#p = Math.pow(_obj$x$y$z.#p, 4);
    _fn$x$y$z = fn().x.y.z, _fn$x$y$z.#p = Math.pow(_fn$x$y$z.#p, 5);
  }
}

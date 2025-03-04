class Repro {
  test() {
    this.f?.();
    this.x.f?.();
    this.x.y.f?.();
    this.x?.f?.();
    this.x?.y.f?.();
    this.x.y?.f?.();
    this.x?.y?.f?.();
    this["x"].f?.();
    (0, this).f?.();
  }
}

const repro = {};
repro.f?.();
repro.x.f?.();
repro.x.y.f?.();
repro.x?.f?.();
repro.x?.y.f?.();
repro.x.y?.f?.();
repro.x?.y?.f?.();
repro["x"].f?.();
(0, repro).f?.()

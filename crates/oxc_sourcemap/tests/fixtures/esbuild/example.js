// index.tsx
import { h as u, Fragment as l, render as c } from "preact";

// counter.tsx
import { h as t, Component as i } from "preact";
import { useState as a } from "preact/hooks";
var n = class extends i {
  constructor(e) {
    super(e);
    this.n = () => this.setState({ t: this.state.t + 1 });
    this.r = () => this.setState({ t: this.state.t - 1 });
    this.state.t = e.e;
  }
  render() {
    return t("div", {
      class: "counter"
    }, t("h1", null, this.props.label), t("p", null, t("button", {
      onClick: this.r
    }, "-"), " ", this.state.t, " ", t("button", {
      onClick: this.n
    }, "+")));
  }
}, s = (r) => {
  let [o, e] = a(r.e);
  return t("div", {
    class: "counter"
  }, t("h1", null, r.o), t("p", null, t("button", {
    onClick: () => e(o - 1)
  }, "-"), " ", o, " ", t("button", {
    onClick: () => e(o + 1)
  }, "+")));
};

// index.tsx
c(
  u(l, null, u(n, {
    o: "Counter 1",
    e: 100
  }), u(s, {
    o: "Counter 2",
    e: 200
  })),
  document.getElementById("root")
);
//# sourceMappingURL=example.js.map

// abc
var \u{61}b\u0063;

// let
var \u{6C}e\u0074;

// static
var st\u{61}\u{074}\u{0069}\u0063;

// yield
var y\u{69}e\u{006C}d;

const obj = {
  // abc
  \u{61}b\u0063,
  // let
  \u{6C}e\u0074,
  // static
  st\u{61}\u{074}\u{0069}\u0063,
  // yield
  y\u{69}e\u{006C}d,
};

const obj2 = {
  // abc: abc
  \u{61}b\u0063: \u{61}b\u0063,
  // let: let
  \u{6C}e\u0074: \u{6C}e\u0074,
  // static: static
  st\u{61}\u{074}\u{0069}\u0063: st\u{61}\u{074}\u{0069}\u0063,
  // yield: yield
  y\u{69}e\u{006C}d: y\u{69}e\u{006C}d,
};

// abc
\u{61}b\u0063: break \u{61}b\u0063;

// let
\u{6C}e\u0074: break \u{6C}e\u0074;

// static
st\u{61}\u{074}\u{0069}\u0063: break st\u{61}\u{074}\u{0069}\u0063;

// yield
y\u{69}e\u{006C}d: break y\u{69}e\u{006C}d;

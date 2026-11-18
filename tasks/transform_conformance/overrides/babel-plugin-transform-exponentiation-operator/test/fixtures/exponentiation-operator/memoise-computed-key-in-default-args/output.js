function fn(o, a = (() => {
  var _o, _g;
  return _o = o, _g = g(), _o[_g] = Math.pow(_o[_g], 2);
})()) {}
var arrow = (o, a = (() => {
  var _o2, _g2;
  return _o2 = o, _g2 = g(), _o2[_g2] = Math.pow(_o2[_g2], 2);
})()) => a;

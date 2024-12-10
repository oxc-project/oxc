var _unboundProp;

let boundProp;

"abc".length && ("abc".length = 1);
"abc"[boundProp] && ("abc"[boundProp] = 2);
"abc"[_unboundProp = unboundProp] && ("abc"[_unboundProp] = 3);

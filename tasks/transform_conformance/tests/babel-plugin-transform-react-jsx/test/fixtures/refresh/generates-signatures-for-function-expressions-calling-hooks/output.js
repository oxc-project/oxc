import { jsx as _jsx } from "react/jsx-runtime";
var _s = $RefreshSig$(), _s2 = $RefreshSig$();
export const A = _s(React.memo(_c2 = _s(React.forwardRef(_c = _s((props, ref) => {
  _s();
  const [foo, setFoo] = useState(0);
  React.useEffect(() => {});
  return _jsx("h1", {
    ref,
    children: foo
  });
}, "useState{[foo, setFoo](0)}\\nuseEffect{}")), "useState{[foo, setFoo](0)}\\nuseEffect{}")), "useState{[foo, setFoo](0)}\\nuseEffect{}");
_c3 = A;
export const B = _s2(React.memo(_c5 = _s2(React.forwardRef(_c4 = _s2(function(props, ref) {
  _s2();
  const [foo, setFoo] = useState(0);
  React.useEffect(() => {});
  return _jsx("h1", {
    ref,
    children: foo
  });
}, "useState{[foo, setFoo](0)}\\nuseEffect{}")), "useState{[foo, setFoo](0)}\\nuseEffect{}")), "useState{[foo, setFoo](0)}\\nuseEffect{}");
_c6 = B;
function hoc() {
  var _s3 = $RefreshSig$();
  return _s3(function Inner() {
    _s3();
    const [foo, setFoo] = useState(0);
    React.useEffect(() => {});
    return _jsx("h1", {
      ref,
      children: foo
    });
  }, "useState{[foo, setFoo](0)}\\nuseEffect{}");
}
export let C = hoc();
var _c, _c2, _c3, _c4, _c5, _c6;
$RefreshReg$(_c, "A$React.memo$React.forwardRef");
$RefreshReg$(_c2, "A$React.memo");
$RefreshReg$(_c3, "A");
$RefreshReg$(_c4, "B$React.memo$React.forwardRef");
$RefreshReg$(_c5, "B$React.memo");
$RefreshReg$(_c6, "B");

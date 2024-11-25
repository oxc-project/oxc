import { jsx as _jsx } from "react/jsx-runtime";
var _s = $RefreshSig$(), _s2 = $RefreshSig$(), _s3 = $RefreshSig$();
function useFancyState() {
  _s();
  const [foo, setFoo] = React.useState(0);
  useFancyEffect();
  return foo;
}
_s(useFancyState, "useState{[foo, setFoo](0)}\\nuseFancyEffect{}", false, function() {
  return [useFancyEffect];
});
const useFancyEffect = () => {
  _s2();
  React.useEffect(() => {});
};
_s2(useFancyEffect, "useEffect{}");
export default function App() {
  _s3();
  const bar = useFancyState();
  return _jsx("h1", { children: bar });
}
_s3(App, "useFancyState{bar}", false, function() {
  return [useFancyState];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");

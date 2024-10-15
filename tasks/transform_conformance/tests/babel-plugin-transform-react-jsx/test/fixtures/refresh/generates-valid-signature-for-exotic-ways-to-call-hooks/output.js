var _s2 = $RefreshSig$();
import FancyHook from "fancy";
import { jsxs as _jsxs } from "react/jsx-runtime";
export default function App() {
  _s2();
  var _s = $RefreshSig$();
  function useFancyState() {
    _s();
    const [foo, setFoo] = React.useState(0);
    useFancyEffect();
    return foo;
  }
  _s(useFancyState, "useState{[foo, setFoo](0)}\\nuseFancyEffect{}", true);
  const bar = useFancyState();
  const baz = FancyHook.useThing();
  React.useState();
  useThePlatform();
  use();
  return _jsxs("h1", { children: [bar, baz] });
}
_s2(App, "useFancyState{bar}\\nuseThing{baz}\\nuseState{}\\nuseThePlatform{}\\nuse{}", true, function() {
  return [FancyHook.useThing];
});
_c = App;
var _c;
$RefreshReg$(_c, "App");

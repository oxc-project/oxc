import A from "./A";
import Store from "./Store";
import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
Store.subscribe();
const Header = styled.div`color: red`;
_c = Header;
const StyledFactory1 = styled("div")`color: hotpink`;
_c2 = StyledFactory1;
const StyledFactory2 = styled("div")({ color: "hotpink" });
_c3 = StyledFactory2;
const StyledFactory3 = styled(A)({ color: "hotpink" });
_c4 = StyledFactory3;
const FunnyFactory = funny.factory``;
let Alias1 = A;
let Alias2 = A.Foo;
const Dict = {};
function Foo() {
  return /* @__PURE__ */ _jsxs("div", { children: [
    /* @__PURE__ */ _jsx(A, {}),
    /* @__PURE__ */ _jsx(B, {}),
    /* @__PURE__ */ _jsx(StyledFactory1, {}),
    /* @__PURE__ */ _jsx(StyledFactory2, {}),
    /* @__PURE__ */ _jsx(StyledFactory3, {}),
    /* @__PURE__ */ _jsx(Alias1, {}),
    /* @__PURE__ */ _jsx(Alias2, {}),
    /* @__PURE__ */ _jsx(Header, {}),
    /* @__PURE__ */ _jsx(Dict.X, {})
  ] });
}
_c5 = Foo;
const B = hoc(A);
_c6 = B;
const NotAComponent = wow(A);
_c7 = NotAComponent;
var _c, _c2, _c3, _c4, _c5, _c6, _c7;
$RefreshReg$(_c, "Header");
$RefreshReg$(_c2, "StyledFactory1");
$RefreshReg$(_c3, "StyledFactory2");
$RefreshReg$(_c4, "StyledFactory3");
$RefreshReg$(_c5, "Foo");
$RefreshReg$(_c6, "B");
$RefreshReg$(_c7, "NotAComponent");

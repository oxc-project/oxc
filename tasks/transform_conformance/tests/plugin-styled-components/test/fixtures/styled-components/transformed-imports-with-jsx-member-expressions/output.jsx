import _styled, { default as _styled2, default as _styled3 } from "styled-components";
import React from "react";
import { css } from "styled-components";
import Icons from "./icons";
const someCss = css([" background:purple;"]);
const App1 = () => {
  return <_StyledIcons />;
};
const App2 = () => {
  return <_StyledIconsFoo />;
};
const App3 = () => {
  return <_StyledIconsFooBar />;
};
var _StyledIcons = _styled(Icons)`${someCss}`;
var _StyledIconsFoo = _styled2(Icons.Foo)`${someCss}`;
var _StyledIconsFooBar = _styled3(Icons.Foo.Bar).withConfig({
  displayName: "input___StyledIconsFooBar",
  componentId: "sc-1wxehft-0"
})(["", ""], someCss);

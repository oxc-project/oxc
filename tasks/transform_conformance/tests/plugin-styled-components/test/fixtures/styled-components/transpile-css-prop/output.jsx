import _styled, { default as _styled2, default as _styled3, default as _styled4, default as _styled5, default as _styled6, default as _styled7, default as _styled8, default as _styled9, default as _styled0, default as _styled1, default as _styled10, default as _styled11, default as _styled12, default as _styled13, default as _styled14, default as _styled15, default as _styled16, default as _styled17, default as _styled18, default as _styled19, default as _styled20, default as _styled21, default as _styled22, default as _styled23, default as _styled24, default as _styled25, default as _styled26 } from "styled-components";
/*
 * Basic fixtures
 */

const StaticString = p => <_StyledP>A</_StyledP>;
const StaticTemplate = p => <_StyledP2>
    A
  </_StyledP2>;
const ObjectProp = p => <_StyledP3>A</_StyledP3>;
const NoChildren = p => <_StyledP4 />;
const CssHelperProp = p => <_StyledP5>
    A
  </_StyledP5>;

/*
 * Dynamic prop
 */

const CustomComp = p => <_StyledParagraph>H</_StyledParagraph>;
const DynamicProp = p => <_StyledP6 $_css={props.cssText}>H</_StyledP6>;
const LocalInterpolation = p => <_StyledP7 $_css2={props.bg}>
    H
  </_StyledP7>;
const FuncInterpolation = p => <_StyledP8>
    H
  </_StyledP8>;
const radius = 10;
const GlobalInterpolation = p => <_StyledP9>
    H
  </_StyledP9>;
const LocalCssHelperProp = p => <_StyledP0 $_css3={p.color}>
    A
  </_StyledP0>;
const DynamicCssHelperProp = p => <_StyledP1>
    A
  </_StyledP1>;
const CustomCompWithDot = p => <_StyledButtonGhost>H</_StyledButtonGhost>;
const NestedCompWithDot = p => <_StyledButtonGhostNew>H</_StyledButtonGhostNew>;
const CustomCompWithDotLowerCase = p => <_StyledButtonGhost2>H</_StyledButtonGhost2>;
const CustomElement = p => <_StyledButtonGhost3>H</_StyledButtonGhost3>;

/* styled component defined after function it's used in */

const EarlyUsageComponent = p => <_StyledThing />;
const Thing3 = styled.div`
  color: blue;
`;
var _StyledThing5 = _styled20(Thing3)(p => ({
  [p.$_css8]: {
    color: 'red'
  }
}));
var _StyledThing4 = _styled19(Thing3)(p => ({
  [p.$_css7]: {
    color: 'red'
  }
}));
var _StyledThing3 = _styled18(Thing3)(p => ({
  [p.$_css6]: {
    color: 'red'
  }
}));
var _StyledThing2 = _styled17(Thing3)(p => ({
  color: p.$_css5
}));
var _StyledThing = _styled15(Thing3)`color: red;`;
const ObjectInterpolation = p => {
  const theme = useTheme();
  return <_StyledP10 $_css4={theme.colors.red}>
      H
    </_StyledP10>;
};
const ObjectInterpolationCustomComponent = p => {
  const theme = useTheme();
  return <_StyledThing2 $_css5={theme.colors.red}>
      H
    </_StyledThing2>;
};
const ObjectInterpolationInKey = p => {
  const theme = useTheme();
  return <_StyledThing3 $_css6={theme.breakpoints.md}>
      H
    </_StyledThing3>;
};
const ObjectFnInterpolationInKey = p => {
  const theme = useTheme();
  return <_StyledThing4 $_css7={theme.breakpoints.md()}>
      H
    </_StyledThing4>;
};
const ObjectFnSimpleInterpolationInKey = p => {
  const foo = '@media screen and (max-width: 600px)';
  return <_StyledThing5 $_css8={foo}>
      H
    </_StyledThing5>;
};
const ObjectPropMixedInputs = p => {
  const color = 'red';
  return <_StyledP11 $_css9={p.background} $_css0={color} $_css1={globalVar} $_css10={getAfterValue()}>
      A
    </_StyledP11>;
};
const ObjectPropWithSpread = () => {
  const css = {
    color: 'red'
  };
  const playing = true;
  return <_StyledDiv $_css11={css} $_css12={playing ? {
    opacity: 0,
    bottom: '-100px'
  } : {}} />;
};
const ObjectInterpolationLogical = ({
  bg,
  content,
  height,
  width,
  ...p
}) => {
  return <_StyledP12 {...p} $_css13={bg || 'red'} $_css14={height ?? '100%'} $_css15={width ? `${width}px` : '100%'} $_css16={content}>
      H
    </_StyledP12>;
};
const ObjectInterpolationMember = p => {
  const theme = useTheme();
  const color = 'red';
  return <_StyledP13 $_css17={theme.colors[color]}>
      H
    </_StyledP13>;
};
const RenderPropComponentCSSProp = () => {
  return <RenderPropComponent>
      {() => <_StyledDiv2 />}
    </RenderPropComponent>;
};
const RenderPropComponentSpread = props => {
  return <RenderPropComponent>
      {() => <div {...props.derivedProps} />}
    </RenderPropComponent>;
};
const RenderPropComponentSpreadCSSProp = props => {
  return <RenderPropComponent>
      {() => <_StyledDiv3 {...props.derivedProps} />}
    </RenderPropComponent>;
};
var _StyledP = _styled("p")`flex: 1;`;
var _StyledP2 = _styled2("p")`
      flex: 1;
    `;
var _StyledP3 = _styled3("p")({
  color: 'blue'
});
var _StyledP4 = _styled4("p")`flex: 1;`;
var _StyledP5 = _styled5("p")`
      color: blue;
    `;
var _StyledParagraph = _styled6(Paragraph)`flex: 1`;
var _StyledP6 = _styled7("p")`${p => p.$_css}`;
var _StyledP7 = _styled8("p")`
      background: ${p => p.$_css2};
    `;
var _StyledP8 = _styled9("p")`
      color: ${props => props.theme.a};
    `;
var _StyledP9 = _styled0("p")`
      border-radius: ${radius}px;
    `;
var _StyledP0 = _styled1("p")`
      color: ${p => p.$_css3};
    `;
var _StyledP1 = _styled10("p")`
      color: ${props => props.theme.color};
    `;
var _StyledButtonGhost = _styled11(Button.Ghost)`flex: 1`;
var _StyledButtonGhostNew = _styled12(Button.Ghost.New)`flex: 1`;
var _StyledButtonGhost2 = _styled13(button.ghost)`flex: 1`;
var _StyledButtonGhost3 = _styled14("button-ghost")`flex: 1`;
var _StyledP10 = _styled16("p")(p => ({
  color: p.$_css4
}));
var _StyledP11 = _styled21("p")(p => ({
  background: p.$_css9,
  color: p.$_css0,
  textAlign: 'left',
  '::before': {
    content: p.$_css1
  },
  '::after': {
    content: p.$_css10
  }
}));
var _StyledDiv = _styled22("div")(p => ({
  ...p.$_css11,
  ...p.$_css12
}));
var _StyledP12 = _styled23("p")(p => ({
  background: p.$_css13,
  height: p.$_css14,
  width: p.$_css15,
  '::before': {
    content: p.$_css16
  }
}));
var _StyledP13 = _styled24("p")(p => ({
  color: p.$_css17
}));
var _StyledDiv2 = _styled25("div")`
            color: black;
          `;
var _StyledDiv3 = _styled26("div")`
            color: black;
          `;

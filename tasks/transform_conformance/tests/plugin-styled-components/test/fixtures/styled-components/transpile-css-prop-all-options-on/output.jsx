import styled, { default as _styled, default as _styled2, default as _styled3, default as _styled4, default as _styled5, default as _styled6, default as _styled7, default as _styled8, default as _styled9, default as _styled0, default as _styled1, default as _styled10, default as _styled11, default as _styled12, default as _styled13, default as _styled14, default as _styled15, default as _styled16, default as _styled17, default as _styled18, default as _styled19, default as _styled20, default as _styled21, default as _styled22, default as _styled23, default as _styled24, default as _styled25, default as _styled26 } from 'styled-components';
import SomeComponent from '../SomeComponentPath';
const {
  SomeOtherComponent
} = require('../SomeOtherComponentPath');

/**
 * control
 */
var _StyledSomeOtherComponent = _styled20(SomeOtherComponent)`color: red;`;
const Thing = styled.div`
  color: red;
`;
const Thing2 = styled(Thing)`
  background: blue;
`;

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
const globalVar = '"foo"';
const getAfterValue = () => '"bar"';
const ObjectPropMixedInputs = p => {
  const color = 'red';
  return <_StyledP10 $_css4={p.background} $_css5={color} $_css6={globalVar} $_css7={getAfterValue()}>
      A
    </_StyledP10>;
};
const SpreadObjectPropMixedInputs = p => {
  const color = 'red';
  return <_StyledP11 $_css8={globalVar} $_css9={getAfterValue()} $_css0={globalVar} $_css1={getAfterValue()} $_css10={p.background} $_css11={globalVar} $_css12={getAfterValue()}>
      A
    </_StyledP11>;
};

/* styled component defined after function it's used in */

const EarlyUsageComponent = p => <_StyledThing />;
const Thing3 = styled.div`
  color: blue;
`;
var _StyledThing6 = _styled25(Thing3)(p => ({
  [p.$_css17]: {
    color: 'red'
  }
}));
var _StyledThing5 = _styled24(Thing3)(p => ({
  [p.$_css16]: {
    color: 'red'
  }
}));
var _StyledThing4 = _styled23(Thing3)(p => ({
  [p.$_css15]: {
    color: 'red'
  }
}));
var _StyledThing3 = _styled22(Thing3)(p => ({
  color: p.$_css14
}));
var _StyledThing = _styled17(Thing3)`color: red;`;
const EarlyUsageComponent2 = p => <_StyledThing2 />;
function Thing4(props) {
  return <div {...props} />;
}

/* insert before usage for non-local scope styled HOC targets */
var _StyledThing2 = _styled18(Thing4)`color: red;`;
const ImportedComponentUsage = p => <_StyledSomeComponent />;
const RequiredComponentUsage = p => <_StyledSomeOtherComponent />;
const ObjectInterpolation = p => {
  const theme = useTheme();
  return <_StyledP12 $_css13={theme.colors.red}>
      H
    </_StyledP12>;
};
const ObjectInterpolationCustomComponent = p => {
  const theme = useTheme();
  return <_StyledThing3 $_css14={theme.colors.red}>
      H
    </_StyledThing3>;
};
const ObjectInterpolationInKey = p => {
  const theme = useTheme();
  return <_StyledThing4 $_css15={theme.breakpoints.md}>
      H
    </_StyledThing4>;
};
const ObjectFnInterpolationInKey = p => {
  const theme = useTheme();
  return <_StyledThing5 $_css16={theme.breakpoints.md()}>
      H
    </_StyledThing5>;
};
const ObjectFnSimpleInterpolationInKey = p => {
  const foo = '@media screen and (max-width: 600px)';
  return <_StyledThing6 $_css17={foo}>
      H
    </_StyledThing6>;
};
const ObjectPropWithSpread = () => {
  const css = {
    color: 'red'
  };
  const playing = true;
  return <_StyledDiv $_css18={css} $_css19={playing ? {
    opacity: 0,
    bottom: '-100px'
  } : {}} />;
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
var _StyledP10 = _styled15("p")(p => ({
  background: p.$_css4,
  color: p.$_css5,
  textAlign: 'left',
  '::before': {
    content: p.$_css6
  },
  '::after': {
    content: p.$_css7
  }
}));
var _StyledP11 = _styled16("p")(p => ({
  ...{
    '::before': {
      content: p.$_css8
    },
    '::after': {
      content: p.$_css9
    },
    ...{
      '::before': {
        content: p.$_css0
      },
      '::after': {
        content: p.$_css1
      }
    }
  },
  background: p.$_css10,
  textAlign: 'left',
  '::before': {
    content: p.$_css11
  },
  '::after': {
    content: p.$_css12
  }
}));
var _StyledSomeComponent = _styled19(SomeComponent)`color: red;`;
var _StyledP12 = _styled21("p")(p => ({
  color: p.$_css13
}));
var _StyledDiv = _styled26("div").withConfig({
  displayName: "input___StyledDiv",
  componentId: "sc-7evkve-0"
})(p => ({
  ...p.$_css18,
  ...p.$_css19
}));

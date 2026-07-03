# externals/prettier/js/multiparser-css/issue-5697.js

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -10,8 +10,9 @@
                 align-items: center;
                 justify-content: center;
                 text-align: center;`
       : ""}
-  @media (max-width: ${(props) => (props.noBreakPoint ? "0" : constants.layout.breakpoint.break1)}px) {
+  @media (max-width: ${(props) =>
+    props.noBreakPoint ? "0" : constants.layout.breakpoint.break1}px) {
     font-size: 2em;
   }
 `;

`````

### Actual (oxfmt)

`````js
const StyledH1 = styled.div`
  font-size: 2.5em;
  font-weight: ${(props) => (props.strong ? 500 : 100)};
  font-family: ${constants.text.displayFont.fontFamily};
  letter-spacing: ${(props) => (props.light ? "0.04em" : 0)};
  color: ${(props) => props.textColor};
  ${(props) =>
    props.center
      ? ` display: flex;
                align-items: center;
                justify-content: center;
                text-align: center;`
      : ""}
  @media (max-width: ${(props) =>
    props.noBreakPoint ? "0" : constants.layout.breakpoint.break1}px) {
    font-size: 2em;
  }
`;

`````

### Expected (prettier)

`````js
const StyledH1 = styled.div`
  font-size: 2.5em;
  font-weight: ${(props) => (props.strong ? 500 : 100)};
  font-family: ${constants.text.displayFont.fontFamily};
  letter-spacing: ${(props) => (props.light ? "0.04em" : 0)};
  color: ${(props) => props.textColor};
  ${(props) =>
    props.center
      ? ` display: flex;
                align-items: center;
                justify-content: center;
                text-align: center;`
      : ""}
  @media (max-width: ${(props) => (props.noBreakPoint ? "0" : constants.layout.breakpoint.break1)}px) {
    font-size: 2em;
  }
`;

`````

## Option 2

`````json
{"printWidth":100}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -10,8 +10,9 @@
                 align-items: center;
                 justify-content: center;
                 text-align: center;`
       : ""}
-  @media (max-width: ${(props) => (props.noBreakPoint ? "0" : constants.layout.breakpoint.break1)}px) {
+  @media (max-width: ${(props) =>
+    props.noBreakPoint ? "0" : constants.layout.breakpoint.break1}px) {
     font-size: 2em;
   }
 `;

`````

### Actual (oxfmt)

`````js
const StyledH1 = styled.div`
  font-size: 2.5em;
  font-weight: ${(props) => (props.strong ? 500 : 100)};
  font-family: ${constants.text.displayFont.fontFamily};
  letter-spacing: ${(props) => (props.light ? "0.04em" : 0)};
  color: ${(props) => props.textColor};
  ${(props) =>
    props.center
      ? ` display: flex;
                align-items: center;
                justify-content: center;
                text-align: center;`
      : ""}
  @media (max-width: ${(props) =>
    props.noBreakPoint ? "0" : constants.layout.breakpoint.break1}px) {
    font-size: 2em;
  }
`;

`````

### Expected (prettier)

`````js
const StyledH1 = styled.div`
  font-size: 2.5em;
  font-weight: ${(props) => (props.strong ? 500 : 100)};
  font-family: ${constants.text.displayFont.fontFamily};
  letter-spacing: ${(props) => (props.light ? "0.04em" : 0)};
  color: ${(props) => props.textColor};
  ${(props) =>
    props.center
      ? ` display: flex;
                align-items: center;
                justify-content: center;
                text-align: center;`
      : ""}
  @media (max-width: ${(props) => (props.noBreakPoint ? "0" : constants.layout.breakpoint.break1)}px) {
    font-size: 2em;
  }
`;

`````

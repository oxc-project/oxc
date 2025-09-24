import * as styled from 'styled-components';
const css = styled.css`
  background: black;
`;
const GlobalStyle = styled.createGlobalStyle`
  html {
    background: black;
  }
`;
const Test = styled.default.div.withConfig({
  displayName: "input__Test",
  componentId: "test-namespace__sc-zljhv7-0"
})`color:red;`;
const before = styled.default.div.withConfig({
  displayName: "input__before",
  componentId: "test-namespace__sc-zljhv7-1"
})`color:blue;`;
styled.default.div.withConfig({
  displayName: "input",
  componentId: "test-namespace__sc-zljhv7-2"
})``;
export default styled.default.button.withConfig({
  displayName: "input",
  componentId: "test-namespace__sc-zljhv7-3"
})``;

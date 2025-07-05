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
  componentId: "test-namespace__sc-3rfj0a-0"
})`color:red;`;
const before = styled.default.div.withConfig({
  displayName: "input__before",
  componentId: "test-namespace__sc-3rfj0a-1"
})`color:blue;`;
styled.default.div.withConfig({
  displayName: "code",
  componentId: "test-namespace__sc-3rfj0a-2"
})``;
export default styled.default.button.withConfig({
  displayName: "code",
  componentId: "test-namespace__sc-3rfj0a-3"
})``;

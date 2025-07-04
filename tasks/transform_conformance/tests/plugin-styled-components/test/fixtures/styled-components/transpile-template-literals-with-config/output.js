import styled, { css, createGlobalStyle } from 'styled-components';
const Named = styled.div.withConfig({
  displayName: "input__Named"
})`
  width: 100%;
`;
const NamedWithInterpolation = styled.div.withConfig({
  displayName: "input__NamedWithInterpolation"
})`
  color: ${color => props.color};
`;
const Wrapped = styled(Inner).withConfig({
  displayName: "input__Wrapped"
})`
  color: red;
`;
const Foo = styled.div.withConfig({
  displayName: "input__Foo"
})({
  color: 'green'
});
const style = css`
  background: green;
`;
const GlobalStyle = createGlobalStyle`
  html {
    background: silver;
  }
`;

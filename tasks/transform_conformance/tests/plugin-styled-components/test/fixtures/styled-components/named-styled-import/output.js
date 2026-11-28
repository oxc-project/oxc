import { styled } from "styled-components";
const Test = styled.div.withConfig({
  displayName: "Test",
  componentId: "sc-kjwefy-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-kjwefy-1"
})`` : styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-kjwefy-2"
})``;
const styles = { One: styled.div.withConfig({
  displayName: "One",
  componentId: "sc-kjwefy-3"
})`` };
let Component;
Component = styled.div.withConfig({
  displayName: "Component",
  componentId: "sc-kjwefy-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-kjwefy-5"
})``;

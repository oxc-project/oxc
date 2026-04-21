import { styled } from "styled-components";
const Test = styled.div.withConfig({
  displayName: "Test",
  componentId: "sc-yes67e-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-yes67e-1"
})`` : styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-yes67e-2"
})``;
const styles = { One: styled.div.withConfig({
  displayName: "One",
  componentId: "sc-yes67e-3"
})`` };
let Component;
Component = styled.div.withConfig({
  displayName: "Component",
  componentId: "sc-yes67e-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-yes67e-5"
})``;

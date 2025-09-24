import { styled } from '../../relative/example';
const Test = styled.div.withConfig({
  displayName: "Test",
  componentId: "sc-elhbfv-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-elhbfv-1"
})`` : styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-elhbfv-2"
})``;
const styles = {
  One: styled.div.withConfig({
    displayName: "One",
    componentId: "sc-elhbfv-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  displayName: "Component",
  componentId: "sc-elhbfv-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-elhbfv-5"
})``;

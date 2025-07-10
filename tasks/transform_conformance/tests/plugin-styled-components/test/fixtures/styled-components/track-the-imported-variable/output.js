import s from "styled-components";
const Test = s.div.withConfig({
  displayName: "Test",
  componentId: "sc-4ndp5k-0"
})`width:100%;`;
const Test2 = true ? s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-4ndp5k-1"
})`` : s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-4ndp5k-2"
})``;
const styles = {
  One: s.div.withConfig({
    displayName: "One",
    componentId: "sc-4ndp5k-3"
  })``
};
let Component;
Component = s.div.withConfig({
  displayName: "Component",
  componentId: "sc-4ndp5k-4"
})``;
const WrappedComponent = s(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-4ndp5k-5"
})``;

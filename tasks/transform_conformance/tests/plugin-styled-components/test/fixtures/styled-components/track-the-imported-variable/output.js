import s from "styled-components";
const Test = s.div.withConfig({
  displayName: "Test",
  componentId: "sc-etvu4p-0"
})`width:100%;`;
const Test2 = true ? s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-etvu4p-1"
})`` : s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-etvu4p-2"
})``;
const styles = {
  One: s.div.withConfig({
    displayName: "One",
    componentId: "sc-etvu4p-3"
  })``
};
let Component;
Component = s.div.withConfig({
  displayName: "Component",
  componentId: "sc-etvu4p-4"
})``;
const WrappedComponent = s(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-etvu4p-5"
})``;

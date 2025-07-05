import s from "styled-components";
const Test = s.div.withConfig({
  displayName: "Test",
  componentId: "sc-wyof43-0"
})`width:100%;`;
const Test2 = true ? s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-wyof43-1"
})`` : s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-wyof43-2"
})``;
const styles = {
  One: s.div.withConfig({
    displayName: "One",
    componentId: "sc-wyof43-3"
  })``
};
let Component;
Component = s.div.withConfig({
  displayName: "Component",
  componentId: "sc-wyof43-4"
})``;
const WrappedComponent = s(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-wyof43-5"
})``;
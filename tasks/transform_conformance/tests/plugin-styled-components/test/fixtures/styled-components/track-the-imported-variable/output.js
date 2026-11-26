import s from "styled-components";
const Test = s.div.withConfig({
  displayName: "Test",
  componentId: "sc-1l9si6-0"
})`width:100%;`;
const Test2 = true ? s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-1l9si6-1"
})`` : s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-1l9si6-2"
})``;
const styles = {
  One: s.div.withConfig({
    displayName: "One",
    componentId: "sc-1l9si6-3"
  })``
};
let Component;
Component = s.div.withConfig({
  displayName: "Component",
  componentId: "sc-1l9si6-4"
})``;
const WrappedComponent = s(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-1l9si6-5"
})``;

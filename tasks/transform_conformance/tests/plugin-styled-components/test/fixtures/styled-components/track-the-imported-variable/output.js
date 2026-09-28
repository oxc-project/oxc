import s from "styled-components";
const Test = s.div.withConfig({
  displayName: "Test",
  componentId: "sc-AQsv3U-0"
})`width:100%;`;
const Test2 = true ? s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-AQsv3U-1"
})`` : s.div.withConfig({
  displayName: "Test2",
  componentId: "sc-AQsv3U-2"
})``;
const styles = {
  One: s.div.withConfig({
    displayName: "One",
    componentId: "sc-AQsv3U-3"
  })``
};
let Component;
Component = s.div.withConfig({
  displayName: "Component",
  componentId: "sc-AQsv3U-4"
})``;
const WrappedComponent = s(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-AQsv3U-5"
})``;

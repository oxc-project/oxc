import styled from "styled-components";
const Test = styled.div.withConfig({
  displayName: "use-directory-name__Test",
  componentId: "sc-193y009-0"
})`color:red;`;
const before = styled.div.withConfig({
  displayName: "use-directory-name__before",
  componentId: "sc-193y009-1"
})`color:blue;`;
styled.div.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-193y009-2"
})``;
export default styled.button.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-193y009-3"
})``;
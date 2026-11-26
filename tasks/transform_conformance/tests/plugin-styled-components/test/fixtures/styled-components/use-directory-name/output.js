import styled from "styled-components";
const Test = styled.div.withConfig({
  displayName: "use-directory-name__Test",
  componentId: "sc-kkellz-0"
})`color:red;`;
const before = styled.div.withConfig({
  displayName: "use-directory-name__before",
  componentId: "sc-kkellz-1"
})`color:blue;`;
styled.div.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-kkellz-2"
})``;
export default styled.button.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-kkellz-3"
})``;

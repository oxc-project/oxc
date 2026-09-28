import styled from "styled-components";
const Test = styled.div.withConfig({
  displayName: "use-directory-name__Test",
  componentId: "sc-pguxZU-0"
})`color:red;`;
const before = styled.div.withConfig({
  displayName: "use-directory-name__before",
  componentId: "sc-pguxZU-1"
})`color:blue;`;
styled.div.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-pguxZU-2"
})``;
export default styled.button.withConfig({
  displayName: "use-directory-name",
  componentId: "sc-pguxZU-3"
})``;

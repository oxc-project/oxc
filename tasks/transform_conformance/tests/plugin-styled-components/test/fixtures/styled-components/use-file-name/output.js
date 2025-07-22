import styled from "styled-components";
const Test = styled.div.withConfig({
  displayName: "input__Test",
  componentId: "sc-bccdtg-0"
})`color:red;`;
const before = styled.div.withConfig({
  displayName: "input__before",
  componentId: "sc-bccdtg-1"
})`color:blue;`;
styled.div.withConfig({
  displayName: "input",
  componentId: "sc-bccdtg-2"
})``;
export default styled.button.withConfig({
  displayName: "input",
  componentId: "sc-bccdtg-3"
})``;

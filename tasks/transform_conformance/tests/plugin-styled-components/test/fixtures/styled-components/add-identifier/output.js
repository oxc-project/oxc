import styled from 'styled-components';
const Test = styled.div.withConfig({
  componentId: "sc-80sor8-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  componentId: "sc-80sor8-1"
})`` : styled.div.withConfig({
  componentId: "sc-80sor8-2"
})``;
const styles = {
  One: styled.div.withConfig({
    componentId: "sc-80sor8-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  componentId: "sc-80sor8-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  componentId: "sc-80sor8-5"
})``;

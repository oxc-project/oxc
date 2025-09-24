import styled from 'styled-components';
const Test = styled.div.withConfig({
  componentId: "sc-yv1qfe-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  componentId: "sc-yv1qfe-1"
})`` : styled.div.withConfig({
  componentId: "sc-yv1qfe-2"
})``;
const styles = {
  One: styled.div.withConfig({
    componentId: "sc-yv1qfe-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  componentId: "sc-yv1qfe-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  componentId: "sc-yv1qfe-5"
})``;

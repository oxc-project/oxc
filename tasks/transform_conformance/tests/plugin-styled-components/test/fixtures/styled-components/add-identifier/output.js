import styled from 'styled-components';
const Test = styled.div.withConfig({
  componentId: "sc-1289dod-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  componentId: "sc-1289dod-1"
})`` : styled.div.withConfig({
  componentId: "sc-1289dod-2"
})``;
const styles = {
  One: styled.div.withConfig({
    componentId: "sc-1289dod-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  componentId: "sc-1289dod-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  componentId: "sc-1289dod-5"
})``;
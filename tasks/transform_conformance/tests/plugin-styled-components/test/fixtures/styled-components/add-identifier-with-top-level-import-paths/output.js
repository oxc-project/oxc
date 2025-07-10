import styled from '@xstyled/styled-components';
import unstyled from '@xstyled/styled-components-test';
const Test = styled.div.withConfig({
  componentId: "sc-1mlyrvc-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  componentId: "sc-1mlyrvc-1"
})`` : styled.div.withConfig({
  componentId: "sc-1mlyrvc-2"
})``;
const styles = {
  One: styled.div.withConfig({
    componentId: "sc-1mlyrvc-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  componentId: "sc-1mlyrvc-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  componentId: "sc-1mlyrvc-5"
})``;
const NoTransformComponent = unstyled.div``;

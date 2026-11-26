import styled from 'styled-components';
const Test = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__Test",
  componentId: "sc-mm0ps0-0"
})(["width:100%;"]);
const Test2 = /*#__PURE__*/styled('div').withConfig({
  displayName: "input__Test2",
  componentId: "sc-mm0ps0-1"
})([""]);
const Test3 = true ? styled.div.withConfig({
  displayName: "input__Test3",
  componentId: "sc-mm0ps0-2"
})([""]) : styled.div.withConfig({
  displayName: "input__Test3",
  componentId: "sc-mm0ps0-3"
})([""]);
const styles = {
  One: styled.div.withConfig({
    displayName: "input__One",
    componentId: "sc-mm0ps0-4"
  })([""])
};
let Component;
Component = styled.div.withConfig({
  displayName: "input__Component",
  componentId: "sc-mm0ps0-5"
})([""]);
const WrappedComponent = /*#__PURE__*/styled(Inner).withConfig({
  displayName: "input__WrappedComponent",
  componentId: "sc-mm0ps0-6"
})([""]);
const StyledObjectForm = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__StyledObjectForm",
  componentId: "sc-mm0ps0-7"
})({
  color: red
});
const StyledFunctionForm = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__StyledFunctionForm",
  componentId: "sc-mm0ps0-8"
})(p => ({
  color: p.color || 'red'
}));
const normalFunc = add(5, 3);

import styled from 'styled-components';
const Test = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__Test",
  componentId: "sc-PRXiXb-0"
})(["width:100%;"]);
const Test2 = /*#__PURE__*/styled('div').withConfig({
  displayName: "input__Test2",
  componentId: "sc-PRXiXb-1"
})([""]);
const Test3 = true ? styled.div.withConfig({
  displayName: "input__Test3",
  componentId: "sc-PRXiXb-2"
})([""]) : styled.div.withConfig({
  displayName: "input__Test3",
  componentId: "sc-PRXiXb-3"
})([""]);
const styles = {
  One: styled.div.withConfig({
    displayName: "input__One",
    componentId: "sc-PRXiXb-4"
  })([""])
};
let Component;
Component = styled.div.withConfig({
  displayName: "input__Component",
  componentId: "sc-PRXiXb-5"
})([""]);
const WrappedComponent = /*#__PURE__*/styled(Inner).withConfig({
  displayName: "input__WrappedComponent",
  componentId: "sc-PRXiXb-6"
})([""]);
const StyledObjectForm = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__StyledObjectForm",
  componentId: "sc-PRXiXb-7"
})({
  color: red
});
const StyledFunctionForm = /*#__PURE__*/styled.div.withConfig({
  displayName: "input__StyledFunctionForm",
  componentId: "sc-PRXiXb-8"
})(p => ({
  color: p.color || 'red'
}));
const normalFunc = add(5, 3);

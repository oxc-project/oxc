import styled from 'styled-components';
const Test = styled.div.withConfig({
  displayName: "Test",
  componentId: "sc-mkpq9u-0"
})`width:100%;`;
const Test2 = true ? styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-mkpq9u-1"
})`` : styled.div.withConfig({
  displayName: "Test2",
  componentId: "sc-mkpq9u-2"
})``;
const styles = {
  One: styled.div.withConfig({
    displayName: "One",
    componentId: "sc-mkpq9u-3"
  })``
};
let Component;
Component = styled.div.withConfig({
  displayName: "Component",
  componentId: "sc-mkpq9u-4"
})``;
const WrappedComponent = styled(Inner).withConfig({
  displayName: "WrappedComponent",
  componentId: "sc-mkpq9u-5"
})``;
const WrappedComponent2 = styled.div.withConfig({
  displayName: "WrappedComponent2",
  componentId: "sc-mkpq9u-6"
})({});
const WrappedComponent3 = styled(Inner).withConfig({
  displayName: "WrappedComponent3",
  componentId: "sc-mkpq9u-7"
})({});
const WrappedComponent4 = styled(Inner).attrs(() => ({
  something: 'else'
})).withConfig({
  displayName: "WrappedComponent4",
  componentId: "sc-mkpq9u-8"
})({});
const WrappedComponent5 = styled.div.attrs(() => ({
  something: 'else'
})).withConfig({
  displayName: "WrappedComponent5",
  componentId: "sc-mkpq9u-9"
})({});
const WrappedComponent6 = styled.div.attrs(() => ({
  something: 'else'
})).withConfig({
  displayName: "WrappedComponent6",
  componentId: "sc-mkpq9u-10"
})``;
const WrappedComponent7 = styled.div.withConfig({
  shouldForwardProp: () => { },
  displayName: "WrappedComponent7",
  componentId: "sc-mkpq9u-11"
})({});
const WrappedComponent8 = styled.div.withConfig({
  shouldForwardProp: () => { },
  displayName: "WrappedComponent8",
  componentId: "sc-mkpq9u-12"
}).attrs(() => ({
  something: 'else'
}))({});
const WrappedComponent9 = styled.div.attrs(() => ({
  something: 'else'
})).withConfig({
  shouldForwardProp: () => { },
  displayName: "WrappedComponent9",
  componentId: "sc-mkpq9u-13"
})({});

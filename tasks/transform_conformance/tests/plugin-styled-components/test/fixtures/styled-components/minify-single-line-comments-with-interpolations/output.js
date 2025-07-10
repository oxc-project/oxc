import styled from 'styled-components';

const Test1 = styled.div.withConfig({ displayName: "input__Test1", componentId: "sc-clieju-0" })(["width:100%;"]);
const Test2 = styled.div.withConfig({ displayName: "input__Test2", componentId: "sc-clieju-1" })(["width:100%;"]);
const Test3 = styled.div.withConfig({ displayName: "input__Test3", componentId: "sc-clieju-2" })(["width:100%;", ";"], 'red');
const Test4 = styled.div.withConfig({ displayName: "input__Test4", componentId: "sc-clieju-3" })(["width:100%;"]);
const Test5 = styled.div.withConfig({ displayName: "input__Test5", componentId: "sc-clieju-4" })(["width:100%;"]);
const Test6 = styled.div.withConfig({ displayName: "input__Test6", componentId: "sc-clieju-5" })(
  ["background:url(\"https://google.com\");width:100%;", ""],
  'green',
);
const Test7 = styled.div.withConfig({ displayName: "input__Test7", componentId: "sc-clieju-6" })(
  ["background:url(\"https://google.com\");width:", ";", "height:", ";"],
  p => p.props.width,
  'green',
  p => p.props.height,
);

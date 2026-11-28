import styled from 'styled-components';

const Test1 = styled.div.withConfig({ displayName: "input__Test1", componentId: "sc-8ow1fs-0" })(["width:100%;"]);
const Test2 = styled.div.withConfig({ displayName: "input__Test2", componentId: "sc-8ow1fs-1" })(["width:100%;"]);
const Test3 = styled.div.withConfig({ displayName: "input__Test3", componentId: "sc-8ow1fs-2" })(["width:100%;", ";"], 'red');
const Test4 = styled.div.withConfig({ displayName: "input__Test4", componentId: "sc-8ow1fs-3" })(["width:100%;"]);
const Test5 = styled.div.withConfig({ displayName: "input__Test5", componentId: "sc-8ow1fs-4" })(["width:100%;"]);
const Test6 = styled.div.withConfig({ displayName: "input__Test6", componentId: "sc-8ow1fs-5" })(
  ["background:url(\"https://google.com\");width:100%;", ""],
  'green',
);
const Test7 = styled.div.withConfig({ displayName: "input__Test7", componentId: "sc-8ow1fs-6" })(
  ["background:url(\"https://google.com\");width:", ";", " height:", ";"],
  p => p.props.width,
  'green',
  p => p.props.height,
);
const Test8 = styled.div.withConfig({ displayName: "input__Test8", componentId: "sc-8ow1fs-7" })(
  ["width:100%;color:", ";height:", ";"],
  "blue",
  123,
);

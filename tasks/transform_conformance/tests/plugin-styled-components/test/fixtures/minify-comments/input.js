import styled from 'styled-components';

const Button = styled.div`${x}/* ${y} */${z}`;

const Foo = styled.div`
  .a // comment
  { color: red; }
`;

const Bar = styled.div`
  .a // ${123}
  { color: red; }
`;

const Qux = styled.div`
  color: /* blah */ red;
  .a /* blah */ { color: blue; }
  .b {/* blah */color: green; }
`;

const Bing = styled.div`
  color: /* ${123} */ red;
  .a /* ${123} */ { color: blue; }
  .b {/* ${123} */color: green; }
`;

const Bong = styled.div`/* ${123} */color: red;`;

const Doom = styled.div` /* ${123} */ color: red; `;

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
  width/* big */: 1000px;
  .a /* blah */ { color: blue; }
`;

const Bing = styled.div`
  color: /* ${123} */ red;
  width/* ${'big'} */: 1000px;
  .a /* ${123} */ { color: blue; }
`;

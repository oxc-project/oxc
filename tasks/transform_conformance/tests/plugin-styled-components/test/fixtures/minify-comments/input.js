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

import styled from 'styled-components';

// Leading or trailing line breaks
const A = styled.div`\r\ncolor: blue;\r\n`;

// Line breaks in removal position
const B = styled.div`
  color:\r\nblue;
  .a\r\n{\r\n}
`;

// Line breaks in non-removal position
const C = styled.div`thing\r\n:hover;`;

// Line breaks before and after interpolations
const D = styled.div`
  foo\r\n${'blue'}\r\n:blah;
`;

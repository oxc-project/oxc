import styled from '@emotion/styled';
const Box = styled('div', {
  target: 'custom-target',
  label: 'CustomLabel',
  shouldForwardProp: prop => prop !== 'color'
})({ padding: 10 });

import styled from 'styled-components';

const PADDING = 2;

// Test case for whitespace before interpolation
const Button = styled.div`
  padding: 0 ${PADDING}px 0 2px;
`;

// Multiple interpolations with spaces before
const Box = styled.div`
  margin: ${props => props.margin}px ${props => props.margin}px;
  padding: 0 ${PADDING}px;
`;

// Space before interpolation in middle of value
const Container = styled.div`
  width: calc(100% - ${PADDING}px);
`;

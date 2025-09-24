import styled from 'styled-components';
const Named = styled.div(["\n  width: 100%;\n"]);
const NamedWithInterpolation = styled.div(["\n  color: ", ";\n"], color => props.color);
const Wrapped = styled(Inner)(["color: red;"]);

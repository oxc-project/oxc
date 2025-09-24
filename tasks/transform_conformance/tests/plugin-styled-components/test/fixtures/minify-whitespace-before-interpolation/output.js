import styled from 'styled-components';
const PADDING = 2;
const Button = styled.div`padding:0 ${PADDING}px 0 2px;`;
const Box = styled.div`margin:${props => props.margin}px ${props => props.margin}px;padding:0 ${PADDING}px;`;
const Container = styled.div`width:calc(100% - ${PADDING}px);`;
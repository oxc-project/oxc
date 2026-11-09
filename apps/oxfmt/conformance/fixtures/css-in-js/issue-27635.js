const a = css`
${t}-${d}: 1rem;
${a}_${b}: 1rem;
${a}x${b}: 1rem;
${a}--${b}: 1rem;
${a}-${b}-c: 1rem;
${a}-${b}-${c}: 1rem;
`;

const b = styled.div`
color: red;
${t}-${d}: 1rem
`;

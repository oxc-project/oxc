const mobile = "(max-width: 600px)";

export const A = styled.div`
color: red;
@media ${mobile} {
color: blue;
}
`;

export const B = css`
  @media not ${a} { color: blue; }
  @media ${a} and (min-width: 1px) { color: blue; }
  @media ${a}, ${b} { color: blue; }
`;

export const C = createGlobalStyle`
  @import url(x) ${a};
  @import url(x) screen, ${b};
`;

export const D = css`
@media screen and ${mobile} {
color: blue;
}
@media ${a} and ${b} { color: blue; }
@supports ${feature} {
color: blue;
}
@supports not ${feature} { color: blue; }
@media print-${x} { color: blue; }
@media ${a}${b} { color: blue; }
`;

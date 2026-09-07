// DIVERGES: `Xxx.extend` / `Xxx.extend.attr(...)` is not a css-in-js tag, the template stays verbatim;
// see apps/oxfmt/DIVERGENCES.md "styled-extend-tag"
const TomatoButton = Button.extend`
	color  : tomato  ;
`;

Button.extend.attr({})`
border-color : black;
`;

// Multiple embedded languages in one file
const mixedStyles = css`.button{color:red;}`;

const mixedStyled = styled.button`padding:10px;`;

const mixedQuery = gql`query{users{name}}`;

const mixedTemplate = html`<div><h1>Title</h1></div>`;

const mixedDocs = md`#Documentation
This is **important**.`;

// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed
const empty = css``;
const empty2 = styled`
`;
const empty3 = styled.div` `;
const empty4 = gql`   `;
const empty5 = html`

`;
const empty6 = md`

`;

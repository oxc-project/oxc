// Multiple embedded languages in one file
const mixedStyles = css`.button{color:red;}`;

const mixedStyled = styled.button`padding:10px;`;

const mixedQuery = gql`query{users{name}}`;

const mixedTemplate = html`<div><h1>Title</h1></div>`;

const mixedDocs = md`#Documentation
This is **important**.`;

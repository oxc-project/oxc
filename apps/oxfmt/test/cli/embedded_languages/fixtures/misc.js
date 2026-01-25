// Regular JavaScript - No embedded languages (should not be affected)
function greet(name) {
  return `Hello, ${name}!`;
}

const message = `This is a regular template string`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// prettier-ignore - Should not format
// prettier-ignore
const unformattedCss = css`.button{color:red;background:blue;}`;

const formattedCss = css`.container{display:flex;align-items:center;}`;

// prettier-ignore
const unformattedGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const formattedGql = gql`query GetPosts{posts{title author}}`;

// Unsupported tags - Should not format
const unknown = customTag`This won't be formatted`;

const sqlQuery = sql`SELECT * FROM users WHERE id = 1`;

// Invalid syntax in supported tag - Should not format
const invalidCss = css`
  repeating-linear-gradient(
    0deg,
var(--color),
    transparent 3px
  );
`;

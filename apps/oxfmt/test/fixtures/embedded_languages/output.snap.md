# Exit code
0

# stdout
```
files/index.js Xms

Formatted 1 files.
Finished in Xms on 1 files using X threads.
```

# stderr
```
```

# Formatted Output
```
// ============================================================================
// CSS - Tagged template literals with css and styled tags
// ============================================================================

const styles = css`
  .button {
    color: red;
    background: blue;
    padding: 10px 20px;
  }
  .container {
    display: flex;
    justify-content: center;
  }
`;

const styledComponent = styled`
  background-color: #ffffff;
  border-radius: 4px;
  padding: 8px;
`;

// ============================================================================
// GraphQL - Tagged template literals with gql and graphql tags
// ============================================================================

const query = gql`
  query GetUser($id: ID!) {
    user(id: $id) {
      name
      email
      posts {
        title
      }
    }
  }
`;

const mutation = graphql`
  mutation CreatePost($input: PostInput!) {
    createPost(input: $input) {
      id
      title
    }
  }
`;

// ============================================================================
// HTML - Tagged template literals with html tag
// ============================================================================

const template = html`
  <div class="container">
    <h1>Hello World</h1>
    <p>This is a paragraph with <strong>bold</strong> text.</p>
  </div>
`;

const component = html`
  <button type="button" onclick="handleClick()">Click me</button>
`;

// ============================================================================
// Markdown - Tagged template literals with md and markdown tags
// ============================================================================

const documentation = md`
  #Heading
  This is **bold** and this is _italic_.
  -Item 1
  -Item 2
`;

const readme = markdown`
  ##Installation
  \`\`\`bash
  npm install package
  \`\`\`
`;

// ============================================================================
// Mixed - Multiple embedded languages in one file
// ============================================================================

const mixedStyles = css`
  .button {
    color: red;
  }
`;

const mixedQuery = gql`
  query {
    users {
      name
    }
  }
`;

const mixedTemplate = html`
  <div><h1>Title</h1></div>
`;

const mixedDocs = md`
  #Documentation
  This is **important**.
`;

// ============================================================================
// No Embedded Languages - Regular JavaScript (no tagged templates)
// ============================================================================

function greet(name) {
  return `Hello, ${name}!`;
}

const message = `This is a regular template string`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// ============================================================================
// prettier-ignore - Skip formatting with prettier-ignore comments
// ============================================================================

// prettier-ignore
const unformatted = css`.button{color:red;background:blue;border:1px solid green;}`;

const formattedCss = css`
  .container {
    display: flex;
    align-items: center;
  }
`;

// prettier-ignore
const ignoredGql = gql`query GetUser($id:ID!){user(id:$id){name email}}`;

const normalGql = gql`
  query GetPosts {
    posts {
      title
      author
    }
  }
`;

// ============================================================================
// Unsupported Tags - Tags not recognized by the formatter
// ============================================================================

const unknown = customTag`
  
    
      
        
          
          
          
          
          
          
                      This is some content that won't be formatted
                      because customTag is not recognized.
`;

const sql = sql`
  
    
      
        
          
          
          
          
          
          
                      SELECT * FROM users WHERE id = 1
`;

```

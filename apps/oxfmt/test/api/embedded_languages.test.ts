import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

const cssSource = `\
// Tagged template literals with css and styled tags
const styles = css\`.button{color:red;background:blue;padding:10px 20px;}\`;

const styledComponent = styled\`background-color:#ffffff;border-radius:4px;\`;

// Member expression tags
const cssGlobal = css.global\`.reset{margin:0;padding:0;}\`;

const styledDiv = styled.div\`width:100%;height:100vh;\`;

const styledLink = styled["a"]\`text-decoration:none;color:#007bff;\`;

const styledButton = styled(Button)\`font-size:16px;color:#333;\`;

// CSS prop and styled-jsx
const cssProp = <div css={\`display: flex; align-items: center;\`}>Hello</div>;

const styledJsx = <style jsx>{\`display: flex; align-items: center;\`}</style>;

// Template literals with \${} substitutions
const dynamic = css\`color:\${color};background:\${bg};\`;

const styledWithExpr = styled.button\`font-size:\${size}px;color:\${theme.primary};\`;

// printWidth-aware: long transition values should break across lines
const animated = styled.div\`transition:width \${duration},height \${duration},top \${duration},left \${duration};\`;

const multiProp = css\`
  .card {padding:\${spacing}px;margin:\${margin};border:1px solid \${borderColor};box-shadow:0 2px 4px \${shadowColor};}
\`;

// Multi-line templates with inherited indentation (dedent before formatting)
const documented = styled.div\`
  /**
   * @description This is a documented section
   * @param {number} value - Some value
   */
  padding: 16px;
\`;
`;

const graphqlSource = `\
// Tagged template literals with gql and graphql tags
const query = gql\`query GetUser($id:ID!){user(id:$id){name email}}\`;

const mutation = graphql\`mutation CreatePost($input:PostInput!){createPost(input:$input){id title}}\`;

// graphql() function call - single argument (hugging layout)
const schema = graphql(\`query{users{name email}}\`);

// graphql() function call - multiple arguments
graphql(schema, \`mutation MarkReadNotificationMutation($input:MarkReadNotificationData!){markReadNotification(data:$input){notification{seenState}}}\`)

// graphql() function call - empty
graphql(\`\`);

// Non-target: gql() is NOT recognized as a function call pattern
gql(\`query{users{name}}\`);

// Non-target: other function names
someFunction(\`query{users{name}}\`);
`;

const htmlSource = `\
// Tagged template literals with html tag
const template = html\`<div class="container"><h1>Hello</h1><p>World</p></div>\`;

const component = html\`<button type="button" onclick="handleClick()">Click</button>\`;
`;

const markdownSource = `\
// Tagged template literals with md and markdown tags
const documentation = md\`#Heading
This is **bold**.
-Item 1
-Item 2\`;

const readme = markdown\`##Installation
\\\`\\\`\\\`bash
npm install package
\\\`\\\`\\\`\`;
`;

const angularSource = `\
// Angular @Component decorator - direct template and styles
// Uses Angular-specific syntax: interpolation, directives, bindings
@Component({
    selector: 'app-root',
    template: \`
        <h1>{{    title    }}</h1>
        <div *ngIf="isVisible"    [class.active]="isActive"     (click)="onClick()">
            <span>{{ count     }}</span>
        </div>
        <ul><li *ngFor="let item of items">{{item.name}}</li></ul>
    \`,
    styles: \`h1 { color: blue }\`
})
export class AppComponent1 {}

// Array form styles
@Component({
       selector: 'app-test',
  template: \`<ul>   <li>test</li>
  </ul>
  \`,
  styles: [   \`

 :host {
   color: red;
 }
 div { background: blue
 }
\`

]
})
class     TestComponent {}

// Computed properties - should NOT be formatted
const styles = "foobar";
const template = "foobar";

@Component({
    selector: 'app-computed',
    [template]: \`<h1>{{       hello }}</h1>\`,
    [styles]: \`h1 { color: blue }\`
})
export class AppComponent2 {}
`;

const mixedSource = `\
// Multiple embedded languages in one file
const mixedStyles = css\`.button{color:red;}\`;

const mixedStyled = styled.button\`padding:10px;\`;

const mixedQuery = gql\`query{users{name}}\`;

const mixedTemplate = html\`<div><h1>Title</h1></div>\`;

const mixedDocs = md\`#Documentation
This is **important**.\`;

// Multi-line with blank lines - should preserve blank lines without trailing whitespace
const multilineCSS = css\`
  .foo {
    color: red;
  }

  .bar {
    color: blue;
  }
\`;

const multilineGQL = gql\`
  type Foo {
    name: String!
  }

  type Bar {
    value: Int!
  }
\`;

// Empty - Regular template literals retain newlines and spaces, but embedded ones are condensed
const empty = css\`\`;
const empty2 = styled\`
\`;
const empty3 = styled.div\` \`;
const empty4 = gql\`   \`;
const empty5 = html\`

\`;
const empty6 = md\`

\`;
`;

const miscSource = `\
// Regular JavaScript - No embedded languages (should not be affected)
function greet(name) {
  return \`Hello, \${name}!\`;
}

const message = \`This is a regular template string\`;

class Formatter {
  format(text) {
    return text.trim();
  }
}

// prettier-ignore - Should not format
// prettier-ignore
const unformattedCss = css\`.button{color:red;background:blue;}\`;

const formattedCss = css\`.container{display:flex;align-items:center;}\`;

// prettier-ignore
const unformattedGql = gql\`query GetUser($id:ID!){user(id:$id){name email}}\`;

const formattedGql = gql\`query GetPosts{posts{title author}}\`;

// Unsupported tags - Should not format
const unknown = customTag\`This won't be formatted\`;

const sqlQuery = sql\`SELECT * FROM users WHERE id = 1\`;

// Invalid syntax in supported tag - Should not format
const invalidCss = css\`
  repeating-linear-gradient(
    0deg,
var(--color),
    transparent 3px
  );
\`;
`;

const languages: [string, string][] = [
  ["css.js", cssSource],
  ["graphql.js", graphqlSource],
  ["html.js", htmlSource],
  ["markdown.js", markdownSource],
  ["angular.ts", angularSource],
];

describe("Embedded languages", () => {
  it.each(languages)("should format %s (auto)", async (filename, source) => {
    const result = await format(filename, source);
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it.each(languages)("should not format %s (off)", async (filename, source) => {
    const result = await format(filename, source, { embeddedLanguageFormatting: "off" });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  it.each(languages)("should format %s with CRLF", async (filename, source) => {
    const result = await format(filename, source, {
      endOfLine: "crlf",
      embeddedLanguageFormatting: "auto",
    });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchSnapshot();
  });

  describe("Misc", () => {
    it("should format multiple embedded languages in one file", async () => {
      const result = await format("mixed.js", mixedSource);
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should not format regular templates, prettier-ignore, unsupported tags, and invalid syntax", async () => {
      const result = await format("misc.js", miscSource);
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });
});

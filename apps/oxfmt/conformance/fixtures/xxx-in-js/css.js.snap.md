# Input

`````js
// Tagged template literals with css and styled tags
const styles = css`.button{color:red;background:blue;padding:10px 20px;}`;

const styledComponent = styled`background-color:#ffffff;border-radius:4px;`;

// Member expression tags
const cssGlobal = css.global`.reset{margin:0;padding:0;}`;

const styledDiv = styled.div`width:100%;height:100vh;`;

const styledLink = styled["a"]`text-decoration:none;color:#007bff;`;

const styledButton = styled(Button)`font-size:16px;color:#333;`;

// CSS prop and styled-jsx
const cssProp = <div css={`display: flex; align-items: center;`}>Hello</div>;

const styledJsx = <style jsx>{`display: flex; align-items: center;`}</style>;

// Template literals with ${} substitutions
const dynamic = css`color:${color};background:${bg};`;

const styledWithExpr = styled.button`font-size:${size}px;color:${theme.primary};`;

// printWidth-aware: long transition values should break across lines
const animated = styled.div`transition:width ${duration},height ${duration},top ${duration},left ${duration};`;

const multiProp = css`
  .card {padding:${spacing}px;margin:${margin};border:1px solid ${borderColor};box-shadow:0 2px 4px ${shadowColor};}
`;

// Multi-line templates with inherited indentation (dedent before formatting)
const documented = styled.div`
  /**
   * @description This is a documented section
   * @param {number} value - Some value
   */
  padding: 16px;
`;

`````

# Output

## `{}`

`````js
// Tagged template literals with css and styled tags
const styles = css`
  .button {
    color: red;
    background: blue;
    padding: 10px 20px;
  }
`;

const styledComponent = styled`
  background-color: #ffffff;
  border-radius: 4px;
`;

// Member expression tags
const cssGlobal = css.global`
  .reset {
    margin: 0;
    padding: 0;
  }
`;

const styledDiv = styled.div`
  width: 100%;
  height: 100vh;
`;

const styledLink = styled["a"]`
  text-decoration: none;
  color: #007bff;
`;

const styledButton = styled(Button)`
  font-size: 16px;
  color: #333;
`;

// CSS prop and styled-jsx
const cssProp = (
  <div
    css={`
      display: flex;
      align-items: center;
    `}
  >
    Hello
  </div>
);

const styledJsx = (
  <style jsx>{`
    display: flex;
    align-items: center;
  `}</style>
);

// Template literals with ${} substitutions
const dynamic = css`
  color: ${color};
  background: ${bg};
`;

const styledWithExpr = styled.button`
  font-size: ${size}px;
  color: ${theme.primary};
`;

// printWidth-aware: long transition values should break across lines
const animated = styled.div`
  transition:
    width ${duration},
    height ${duration},
    top ${duration},
    left ${duration};
`;

const multiProp = css`
  .card {
    padding: ${spacing}px;
    margin: ${margin};
    border: 1px solid ${borderColor};
    box-shadow: 0 2px 4px ${shadowColor};
  }
`;

// Multi-line templates with inherited indentation (dedent before formatting)
const documented = styled.div`
  /**
   * @description This is a documented section
   * @param {number} value - Some value
   */
  padding: 16px;
`;

`````

## `{"embeddedLanguageFormatting":"off"}`

`````js
// Tagged template literals with css and styled tags
const styles = css`.button{color:red;background:blue;padding:10px 20px;}`;

const styledComponent = styled`background-color:#ffffff;border-radius:4px;`;

// Member expression tags
const cssGlobal = css.global`.reset{margin:0;padding:0;}`;

const styledDiv = styled.div`width:100%;height:100vh;`;

const styledLink = styled["a"]`text-decoration:none;color:#007bff;`;

const styledButton = styled(Button)`font-size:16px;color:#333;`;

// CSS prop and styled-jsx
const cssProp = <div css={`display: flex; align-items: center;`}>Hello</div>;

const styledJsx = <style jsx>{`display: flex; align-items: center;`}</style>;

// Template literals with ${} substitutions
const dynamic = css`color:${color};background:${bg};`;

const styledWithExpr = styled.button`font-size:${size}px;color:${theme.primary};`;

// printWidth-aware: long transition values should break across lines
const animated = styled.div`transition:width ${duration},height ${duration},top ${duration},left ${duration};`;

const multiProp = css`
  .card {padding:${spacing}px;margin:${margin};border:1px solid ${borderColor};box-shadow:0 2px 4px ${shadowColor};}
`;

// Multi-line templates with inherited indentation (dedent before formatting)
const documented = styled.div`
  /**
   * @description This is a documented section
   * @param {number} value - Some value
   */
  padding: 16px;
`;

`````

## `{"endOfLine":"crlf"}`

`````js
// Tagged template literals with css and styled tags<CRLF>
const styles = css`<CRLF>
  .button {<CRLF>
    color: red;<CRLF>
    background: blue;<CRLF>
    padding: 10px 20px;<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const styledComponent = styled`<CRLF>
  background-color: #ffffff;<CRLF>
  border-radius: 4px;<CRLF>
`;<CRLF>
<CRLF>
// Member expression tags<CRLF>
const cssGlobal = css.global`<CRLF>
  .reset {<CRLF>
    margin: 0;<CRLF>
    padding: 0;<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
const styledDiv = styled.div`<CRLF>
  width: 100%;<CRLF>
  height: 100vh;<CRLF>
`;<CRLF>
<CRLF>
const styledLink = styled["a"]`<CRLF>
  text-decoration: none;<CRLF>
  color: #007bff;<CRLF>
`;<CRLF>
<CRLF>
const styledButton = styled(Button)`<CRLF>
  font-size: 16px;<CRLF>
  color: #333;<CRLF>
`;<CRLF>
<CRLF>
// CSS prop and styled-jsx<CRLF>
const cssProp = (<CRLF>
  <div<CRLF>
    css={`<CRLF>
      display: flex;<CRLF>
      align-items: center;<CRLF>
    `}<CRLF>
  ><CRLF>
    Hello<CRLF>
  </div><CRLF>
);<CRLF>
<CRLF>
const styledJsx = (<CRLF>
  <style jsx>{`<CRLF>
    display: flex;<CRLF>
    align-items: center;<CRLF>
  `}</style><CRLF>
);<CRLF>
<CRLF>
// Template literals with ${} substitutions<CRLF>
const dynamic = css`<CRLF>
  color: ${color};<CRLF>
  background: ${bg};<CRLF>
`;<CRLF>
<CRLF>
const styledWithExpr = styled.button`<CRLF>
  font-size: ${size}px;<CRLF>
  color: ${theme.primary};<CRLF>
`;<CRLF>
<CRLF>
// printWidth-aware: long transition values should break across lines<CRLF>
const animated = styled.div`<CRLF>
  transition:<CRLF>
    width ${duration},<CRLF>
    height ${duration},<CRLF>
    top ${duration},<CRLF>
    left ${duration};<CRLF>
`;<CRLF>
<CRLF>
const multiProp = css`<CRLF>
  .card {<CRLF>
    padding: ${spacing}px;<CRLF>
    margin: ${margin};<CRLF>
    border: 1px solid ${borderColor};<CRLF>
    box-shadow: 0 2px 4px ${shadowColor};<CRLF>
  }<CRLF>
`;<CRLF>
<CRLF>
// Multi-line templates with inherited indentation (dedent before formatting)<CRLF>
const documented = styled.div`<CRLF>
  /**<CRLF>
   * @description This is a documented section<CRLF>
   * @param {number} value - Some value<CRLF>
   */<CRLF>
  padding: 16px;<CRLF>
`;<CRLF>

`````

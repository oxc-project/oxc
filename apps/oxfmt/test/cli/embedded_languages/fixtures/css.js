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


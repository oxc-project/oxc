// prettier/prettier#20355
html`
  <html>
    <body>
      <form>
        <textarea>
${Match.valueTags(form, {
  EmptyForm: () => '',
  InvalidForm: () => '',
  CompletedForm: () => '',
})}</textarea>
      </form>
    </body>
  </html>
`;

const indented = html`
  <form>
    <pre>
  ${foo({
    x: 1,
  })}</pre>
  </form>
`;

const afterText = html`<pre>
a ${foo({
  x: 1,
})} b</pre>`;

const sameLine = html`<pre>
  ${a({
    x: 1,
  })} ${b({
    y: 2,
  })}</pre>`;

const afterElement = html`<div><pre><b>x</b>${foo({
  x: 1,
})}</pre></div>`;

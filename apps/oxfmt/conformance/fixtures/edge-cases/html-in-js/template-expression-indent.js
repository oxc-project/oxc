// prettier/prettier#19725: embedded `${expr}` no longer preserves source indentation
_ = html`
  <div>
                      ${
                        a + //
                        b
                      }
  </div>
`;

// prettier/prettier#19518: nested embeds were not idempotent
const t = html`
  <ol>
    ${items.map(
      (entry) => html`
        <li>
          ${entry.children
            ? html`
                <ol>
                  ${entry.children.map(
                    (child) => html`<li>${child.title}</li>`,
                  )}
                </ol>
              `
            : entry.title}
        </li>
      `,
    )}
  </ol>
`;

export function foo() {
  return html`
    <div>
              <pre>${JSON.stringify({
                  a: 1,
                  b: 2,
                })}</pre>
    </div>
  `;
}

const a = html`
          ${{
              c: y,
          }}
`;

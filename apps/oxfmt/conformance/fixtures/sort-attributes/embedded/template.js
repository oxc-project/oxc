const markup = html`<div z="last" a="first"></div>`;
const interpolated = html`<div z=${last()} a=${first()}></div>`;

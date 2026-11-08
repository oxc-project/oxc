const markup = html`<div z="last" a="first"></div>`;
const interpolated = html`<div z=${last()} a=${first()}></div>`;
const spread = html`<div z="before" ...${props} a="after"></div>`;
const reverse = html`<div a="before" ...${props} z="after"></div>`;
const dynamicName = html`<div z="before" ${name}="value" a="after"></div>`;
const partialName = html`<div z="before" data-${key}="value" a="after"></div>`;
const separated = html`<div z="before" a="before" ...${props} y="after" b="after"></div>`;

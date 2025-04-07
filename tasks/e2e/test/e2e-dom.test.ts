// @vitest-environment happy-dom
import { describe, expect, test } from 'vitest';

import { getModules } from './utils';

const info = '$type $options';

describe('alpine', async () => {
  const modules = await getModules('alpinejs/dist/', 'module.cjs.js', false);
  test.each(modules)(info, ({ module: { Alpine } }) => {
    expect(Alpine.reactive({ count: 1 }).count).toBe(1);
  });
});

describe('lit', async () => {
  const modules = await getModules('lit/all/', 'lit-all.min.js', true);
  test.each(modules)(info, ({ module: Lit }) => {
    expect(Lit.html`<div>rendered</div>`).toBeDefined();
  });
});

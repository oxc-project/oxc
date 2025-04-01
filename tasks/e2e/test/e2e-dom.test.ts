// @vitest-environment happy-dom
import { describe, expect, test } from 'vitest';

import { getModules } from './utils';

const info = '$type $options';

describe('alpine', async () => {
  const modules = await getModules('alpinejs/dist/', 'module.cjs.js', 'cjs');
  test.each(modules)(info, ({ module: { Alpine } }) => {
    expect(Alpine.reactive({ count: 1 }).count).toBe(1);
  });
});

describe('lit', async () => {
  const modules = await getModules('lit/all/', 'lit-all.min.js', 'esm');
  test.each(modules)(info, ({ module: Lit }) => {
    expect(Lit.html`<div>rendered</div>`).toBeDefined();
  });
});

describe('jquery', async () => {
  const modules = await getModules('jquery/dist/', 'jquery.js', 'cjs');
  test.each(modules)(info, ({ module: jQuery }) => {
    expect(jQuery('<div>rendered</div>').text()).toBe('rendered');
  });
});

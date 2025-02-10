import { createElement } from 'react';
import { describe, expect, test } from 'vitest';

import { getModules } from './utils';

const info = '$type $options';

describe('react', () => {
  const modules = getModules('react/cjs/', 'react.development.js');
  test.each(modules)(info, ({ module: React }) => {
    expect(React.createElement('div', null, 'rendered').type).toBe('div');
  });
});

describe('vue', () => {
  const modules = getModules('vue/dist/', 'vue.cjs.js');
  test.each(modules)(info, ({ module: Vue }) => {
    expect(Vue.createApp()).toBeDefined();
  });
});

describe('antd', () => {
  const modules = getModules('antd/dist/', 'antd.js');
  test.each(modules)(info, ({ module: Antd }) => {
    const e = createElement(Antd.Button, null);
    expect(e.type.__ANT_BUTTON).toBe(true);
  });
});

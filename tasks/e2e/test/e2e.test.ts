import { createElement } from 'react';
import { renderToString } from 'react-dom/server';
import { describe, expect, test } from 'vitest';

import { getModules } from './utils';

const info = '$type $options';

describe('react', async () => {
  const modules = await getModules('react/cjs/', 'react.development.js', false);
  test.each(modules)(info, ({ module: React }) => {
    expect(React.createElement('div', null, 'rendered').type).toBe('div');
  });
});

describe('vue', async () => {
  const modules = await getModules('vue/dist/', 'vue.cjs.js', false);
  test.each(modules)(info, ({ module: Vue }) => {
    expect(Vue.createApp()).toBeDefined();
  });
});

describe('preact', async () => {
  const modules = await getModules('preact/dist/', 'preact.js', false);
  test.each(modules)(info, ({ module: Preact }) => {
    expect(Preact.h('div', null, 'rendered').type).toBe('div');
  });
});

describe('solid', async () => {
  const modules = await getModules('solid-js/dist/', 'solid.cjs', false);
  test.each(modules)(info, ({ module: Solid }) => {
    expect(Solid.createSignal(1)[0]()).toBe(1);
  });
});

describe('antd', async () => {
  const modules = await getModules('antd/dist/', 'antd.js', false);
  test.each(modules)(info, ({ module: Antd }) => {
    const components = [
      'Affix',
      'Alert',
      'Anchor',
      'App',
      'AutoComplete',
      'Avatar',
      'BackTop',
      'Badge',
      'Breadcrumb',
      'Button',
      'Calendar',
      'Card',
      'Carousel',
      'Cascader',
      'Checkbox',
      'Col',
      'Collapse',
      'ColorPicker',
      'ConfigProvider',
      'DatePicker',
      'Descriptions',
      'Divider',
      'Drawer',
      'Dropdown',
      'Empty',
      'Flex',
      'FloatButton',
      'Form',
      'Image',
      'Input',
      'InputNumber',
      'Layout',
      'List',
      'Mentions',
      'Menu',
      'Modal',
      'Pagination',
      'Popconfirm',
      'Popover',
      'Progress',
      'QRCode',
      'Radio',
      'Rate',
      'Result',
      'Row',
      'Segmented',
      'Select',
      'Skeleton',
      'Slider',
      'Space',
      'Spin',
      'Splitter',
      'Statistic',
      'Steps',
      'Switch',
      'Table',
      'Tabs',
      'Tag',
      'TimePicker',
      'Timeline',
      'Tooltip',
      'Tour',
      'Transfer',
      'Tree',
      'TreeSelect',
      'Typography',
      'Upload',
      'Watermark',
    ];
    components.forEach((c) => {
      const Component = Antd[c];
      const e = createElement(Component, null);
      const s = renderToString(e);
      expect(s).toBeTypeOf('string');
    });
  });
});

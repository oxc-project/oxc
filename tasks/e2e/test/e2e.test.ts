import { createElement } from 'react';
import { renderToString } from 'react-dom/server';
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

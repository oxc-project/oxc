import { createElement } from 'react';
import { renderToString } from 'react-dom/server';
import { describe, expect, test } from 'vitest';

import { getModules } from './utils';

const info = '$type $options';

describe('react', async () => {
  const modules = await getModules('react/cjs/', 'react.development.js', 'cjs');
  test.each(modules)(info, ({ module: React }) => {
    expect(React.createElement('div', null, 'rendered').type).toBe('div');
  });
});

describe('vue', async () => {
  const modules = await getModules('vue/dist/', 'vue.cjs.js', 'cjs');
  test.each(modules)(info, ({ module: Vue }) => {
    expect(Vue.createApp()).toBeDefined();
  });
});

describe('preact', async () => {
  const modules = await getModules('preact/dist/', 'preact.js', 'cjs');
  test.each(modules)(info, ({ module: Preact }) => {
    expect(Preact.h('div', null, 'rendered').type).toBe('div');
  });
});

describe('solid', async () => {
  const modules = await getModules('solid-js/dist/', 'solid.cjs', 'cjs');
  test.each(modules)(info, ({ module: Solid }) => {
    expect(Solid.createSignal(1)[0]()).toBe(1);
  });
});

describe('antd', async () => {
  const modules = await getModules('antd/dist/', 'antd.js', 'cjs');
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

describe('lodash', async () => {
  const modules = await getModules('lodash/', 'lodash.js', 'cjs');
  test.each(modules)(info, ({ module: Lodash }) => {
    expect(Lodash.chunk([1, 2, 3], 2)).toEqual([[1, 2], [3]]);
  });
});

describe('es-toolkit', async () => {
  const modules = await getModules('es-toolkit/dist/', 'browser.global.js', 'iife', code => code + '\nwindow._ = _');
  test.each(modules)(info, ({ module: window }) => {
    expect(window._.isObject({})).toBe(true);
  });
});

describe('date-fns', async () => {
  const modules = await getModules('date-fns/', 'cdn.js', 'iife');
  test.each(modules)(info, ({ module: window }) => {
    expect(window.dateFns.format(new Date(2023, 0, 1), 'yyyy-MM-dd')).toBe('2023-01-01');
  });
});

describe('moment', async () => {
  const modules = await getModules('moment/min/', 'moment-with-locales.js', 'cjs');
  test.each(modules)(info, ({ module: Moment }) => {
    expect(Moment('2023-01-01').format('YYYY-MM-DD')).toBe('2023-01-01');
  });
});

describe('dayjs', async () => {
  const modules = await getModules('dayjs/', 'dayjs.min.js', 'cjs');
  test.each(modules)(info, ({ module: Dayjs }) => {
    expect(Dayjs('2023-01-01').format('YYYY-MM-DD')).toBe('2023-01-01');
  });
});

describe('luxon', async () => {
  const modules = await getModules('luxon/build/node/', 'luxon.js', 'cjs');
  test.each(modules)(info, ({ module: Luxon }) => {
    expect(Luxon.DateTime.fromISO('2023-01-01').toFormat('yyyy-MM-dd')).toBe('2023-01-01');
  });
});

describe('zod', async () => {
  const modules = await getModules('zod/lib/', 'index.mjs', 'esm');
  test.each(modules)(info, ({ module: Zod }) => {
    expect(Zod.z.object({ name: Zod.z.string() }).parse({ name: 'John' })).toEqual({ name: 'John' });
  });
});

describe('yup', async () => {
  const modules = await getModules('yup/', 'index.js', 'cjs');
  test.each(modules)(info, ({ module: Yup }) => {
    expect(Yup.object({ name: Yup.string() }).isValidSync({ name: 'John' })).toBe(true);
  });
});

describe('rxjs', async () => {
  const modules = await getModules('rxjs/dist/bundles/', 'rxjs.umd.js', 'cjs');
  test.each(modules)(info, async ({ module: Rxjs }) => {
    expect.assertions(1);
    const result = await new Promise((resolve) => {
      Rxjs.interval(100).pipe(Rxjs.take(3), Rxjs.toArray()).subscribe(resolve);
    });
    expect(result).toEqual([0, 1, 2]);
  });
});

describe('immer', async () => {
  const modules = await getModules('immer/dist/cjs/', 'immer.cjs.development.js', 'cjs');
  test.each(modules)(info, ({ module: Immer }) => {
    expect(
      Immer.produce({ count: 1 }, (draft) => {
        draft.count += 1;
      }).count,
    ).toBe(2);
  });
});

describe('ramda', async () => {
  const modules = await getModules('ramda/dist/', 'ramda.js', 'cjs');
  test.each(modules)(info, ({ module: Ramda }) => {
    expect(Ramda.add(1, 2)).toBe(3);
  });
});

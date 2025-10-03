// Custom compatibility data for features not yet in compat-table
// This file contains manually maintained browser support information

const f = (es) => (item) => {
  item.es = es;
  return item;
};

const customEs2020 = [
  {
    name: 'ExportNamespaceFrom',
    babel: 'transform-export-namespace-from',
    // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export#browser_compatibility
    targets: {
      chrome: '72',
      opera: '60',
      edge: '79',
      firefox: '80',
      safari: '14.1',
      node: '13.2',
      deno: '1.0',
      ios: '14.5',
      samsung: '11.0',
      opera_mobile: '51',
      electron: '5.0',
    },
  },
].map(f('ES2020'));

module.exports = [...customEs2020];

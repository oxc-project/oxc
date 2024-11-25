// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/build-data.js
// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/utils-build-data.js

const fs = require('node:fs');
const envs = require('./compat-table/environments');
const parseEnvsVersions = require('./compat-table/build-utils/parse-envs-versions');
const interpolateAllResults = require('./compat-table/build-utils/interpolate-all-results');
const compareVersions = require('./compat-table/build-utils/compare-versions');
const { addElectronSupportFromChromium } = require('./chromium-to-electron');
const esFeatures = require(`./es-features`);

const environments = [
  'chrome',
  'opera',
  'edge',
  'firefox',
  'safari',
  'node',
  'deno',
  'ie',
  'android',
  'ios',
  // 'phantom',
  'samsung',
  'rhino',
  'opera_mobile',
];

const envsVersions = parseEnvsVersions(envs);

const compatSources = ['es5', 'es6', 'es2016plus', 'esnext'].map(source => {
  const data = require(`./compat-table/data-${source}`);
  interpolateAllResults(data.tests, envs);
  return data;
});

const compatibilityTests = compatSources.flatMap(data =>
  data.tests.flatMap(test => {
    if (!test.subtests) return test;

    return test.subtests.map(subtest =>
      Object.assign({}, subtest, {
        name: test.name + ' / ' + subtest.name,
        group: test.name,
      })
    );
  })
);

const getLowestImplementedVersion = (
  { features },
  env,
  exclude = () => false,
) => {
  const tests = compatibilityTests.filter(test => {
    let ok = features.includes(test.name);
    ok ||= test.group && features.includes(test.group);
    ok ||= features.length === 1 && test.name.startsWith(features[0]);
    ok &&= !exclude(test.name);
    return ok;
  });
  const envTests = tests.map(({ res }) => {
    const versions = envsVersions[env];
    let i = versions.length - 1;

    // Find the last not-implemented version
    for (; i >= 0; i--) {
      const { id } = versions[i];
      // Babel assumes strict mode
      if (res[id] !== true && res[id] !== 'strict') {
        break;
      }
    }

    return envsVersions[env][i + 1];
  });

  if (envTests.length === 0 || envTests.some(t => !t)) return null;

  const result = envTests.reduce((a, b) => {
    return compareVersions(a, b) > 0 ? a : b;
  });

  // NOTE(bng): A number of environments in compat-table changed to
  // include a trailing zero (node10 -> node10_0), so for now stripping
  // it to be consistent
  return result.version.join('.').replace(/\.0$/, '');
};

const expandFeatures = features =>
  features.flatMap(feat => {
    if (feat.includes('/')) return [feat];
    return compatibilityTests
      .map(test => test.name)
      .filter(name => name === feat || name.startsWith(feat + ' / '));
  });

const generateData = (environments, items) => {
  for (const item of items) {
    const targets = {};
    environments.forEach(env => {
      const version = getLowestImplementedVersion({
        features: expandFeatures(item.features),
      }, env);
      if (version) targets[env] = version;
    });
    addElectronSupportFromChromium(targets);

    item.targets = targets;
  }

  return items;
};

const items = generateData(environments, esFeatures);

fs.writeFileSync('./data.json', JSON.stringify(items, null, 2));

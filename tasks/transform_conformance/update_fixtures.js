// Script to amend test fixtures for transforms.
//
// Babel's test fixtures for some plugins are not usable for us because they either use
// `@babel/preset-env` or transforms which we haven't implemented yet (e.g. `@babel/transform-classes`).
//
// This script:
// 1. Removes unsupported options from `options.json` files.
// 2. Transforms the input code with Babel using the updated options, and saves as `output.js`.

// TODO: We follow Babel 8 not Babel 7. So need to:
// 1. Use Babel 8 to transform code.
// 2. Skip the fixtures which are marked "SKIP_babel7plugins_babel8core"
//    (or maybe don't - what does this option mean?)

import { transformFileAsync } from '@babel/core';
import { readdir, readFile, rename, writeFile } from 'fs/promises';
import { join as pathJoin } from 'path';

const PACKAGES = ['babel-plugin-transform-class-properties'];
const FILTER_OUT_PRESETS = ['env'];
const FILTER_OUT_PLUGINS = [
  'transform-classes',
  'transform-block-scoping',
  'transform-destructuring',
];

const PACKAGES_PATH = pathJoin(import.meta.dirname, '../coverage/babel/packages');

// Copied from `@babel/helper-transform-fixture-test-runner`
const EXTERNAL_HELPERS_VERSION = '7.100.0';

for (const packageName of PACKAGES) {
  const dirPath = pathJoin(PACKAGES_PATH, packageName, 'test/fixtures');
  await updateDir(dirPath, {}, false);
}

/**
 * Update fixtures in directory, and its sub-directories.
 * @param {string} dirPath - Path to directory containing fixtures
 * @param {Object} options - Transform options from parent directory
 * @param {boolean} hasChangedOptions - `true` if transform options from parent directory have changed
 * @returns {undefined}
 */
async function updateDir(dirPath, options, hasChangedOptions) {
  const files = await readdir(dirPath, { withFileTypes: true });

  const dirFiles = [];
  let optionsFile, inputFile, outputFile;
  for (const file of files) {
    if (file.isDirectory()) {
      dirFiles.push(file);
    } else if (file.name === 'options.json') {
      optionsFile = file;
    } else if (file.name === 'output.js') {
      outputFile = file;
    } else if (file.name.startsWith('input.')) {
      inputFile = file;
    }
  }

  if (optionsFile) {
    const path = pathJoin(dirPath, optionsFile.name);
    const localOptions = JSON.parse(await readFile(path, 'utf8'));
    if (updateOptions(localOptions)) {
      hasChangedOptions = true;
      const backupPath = pathJoin(dirPath, 'options.original.json');
      await rename(path, backupPath);
      await writeFile(path, JSON.stringify(localOptions, null, 2) + '\n');
    }
    options = mergeOptions(options, localOptions);
  }

  if (outputFile && hasChangedOptions) {
    const inputPath = pathJoin(dirPath, inputFile.name);
    const outputPath = pathJoin(dirPath, outputFile.name);
    const backupOutputPath = pathJoin(dirPath, 'output.original.js');
    await rename(outputPath, backupOutputPath);
    await transform(inputPath, outputPath, options);
  }

  for (const file of dirFiles) {
    const path = pathJoin(dirPath, file.name);
    await updateDir(path, options, hasChangedOptions);
  }
}

/**
 * Remove unsupported presets + plugins from `options`.
 * @param {Object} options - Options object
 * @returns {boolean} - `true` if `options` has been altered.
 */
function updateOptions(options) {
  let hasChangedOptions = false;

  function filter(key, filterOut) {
    if (!options[key]) return;
    options[key] = options[key].filter((plugin) => {
      if (filterOut.includes(getName(plugin))) {
        hasChangedOptions = true;
        return false;
      }
      return true;
    });
    if (options[key].length === 0) delete options[key];
  }

  filter('presets', FILTER_OUT_PRESETS);
  filter('plugins', FILTER_OUT_PLUGINS);

  return hasChangedOptions;
}

/**
 * Merge `options` into `parentOptions`.
 * Returns merged options object. Does not mutate either input.
 * @param {Object} parentOptions - Parent options
 * @param {Object} options - Local options
 * @returns {Object} - Merged options object
 */
function mergeOptions(parentOptions, options) {
  parentOptions = { ...parentOptions };

  function merge(key) {
    if (!options[key]) return;

    if (!parentOptions[key]) {
      parentOptions[key] = options[key];
      return;
    }

    parentOptions[key] = [...parentOptions[key]];

    const parentPluginIndexes = new Map();
    for (const [index, plugin] of parentOptions[key].entries()) {
      parentPluginIndexes.set(getName(plugin), index);
    }

    for (const plugin of options[key]) {
      const pluginName = getName(plugin);
      const parentPluginIndex = parentPluginIndexes.get(pluginName);
      if (parentPluginIndex !== undefined) {
        parentOptions[key][parentPluginIndex] = plugin;
      } else {
        parentOptions[key].push(plugin);
      }
    }
  }

  merge('presets');
  merge('plugins');

  if (options.assumptions) {
    parentOptions.assumptions = { ...parentOptions.assumptions, ...options.assumptions };
  }

  for (const [key, value] of Object.entries(options)) {
    if (key === 'plugins' || key === 'presets' || key === 'assumptions') continue;
    if (Object.hasOwn(parentOptions, key)) throw new Error(`Clash: ${key}`);
    parentOptions[key] = value;
  }

  return parentOptions;
}

/**
 * Transform input with Babel and save to output file.
 * @param {string} inputPath - Path of input file
 * @param {string} outputPath - Path of output file
 * @param {Object} options - Transform options
 * @returns {undefined}
 */
async function transform(inputPath, outputPath, options) {
  options = { ...options, configFile: false, babelrc: false, cwd: import.meta.dirname };
  delete options.SKIP_babel7plugins_babel8core;
  delete options.minNodeVersion;

  function prefixName(plugin, type) {
    if (Array.isArray(plugin)) {
      plugin = [...plugin];
      plugin[0] = `@babel/${type}-${plugin[0]}`;
    } else {
      plugin = `@babel/${type}-${plugin}`;
    }
    return plugin;
  }

  if (options.presets) options.presets = options.presets.map(preset => prefixName(preset, 'preset'));

  options.plugins = (options.plugins || []).map(plugin => prefixName(plugin, 'plugin'));
  options.plugins.push(['@babel/plugin-external-helpers', { helperVersion: EXTERNAL_HELPERS_VERSION }]);

  const { code } = await transformFileAsync(inputPath, options);
  await writeFile(outputPath, code);
}

/**
 * Get name of plugin/preset.
 * @param {string|Array} stringOrArray - Input
 * @returns {string} - Name of plugin/preset
 */
function getName(stringOrArray) {
  if (Array.isArray(stringOrArray)) return stringOrArray[0];
  return stringOrArray;
}

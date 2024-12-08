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
import assert from 'assert';
import { copyFile, readdir, readFile, rename, writeFile } from 'fs/promises';
import { extname, join as pathJoin } from 'path';

const PACKAGES = ['babel-plugin-transform-class-properties'];
const FILTER_OUT_PRESETS = ['env'];
const FILTER_OUT_PLUGINS = [
  'transform-classes',
  'transform-block-scoping',
  'transform-destructuring',
];

const PACKAGES_PATH = pathJoin(import.meta.dirname, '../coverage/babel/packages');
const OVERRIDES_PATH = pathJoin(import.meta.dirname, 'overrides');

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

  const filenames = { options: null, input: null, output: null, exec: null };
  const overrides = { options: false, input: false, output: false, exec: false };

  // Find files in dir
  for (const file of files) {
    const filename = file.name;
    if (file.isDirectory()) {
      dirFiles.push(filename);
    } else {
      const ext = extname(filename),
        type = ext === '' ? filename : filename.slice(0, -ext.length);
      if (Object.hasOwn(filenames, type)) filenames[type] = filename;
    }
  }

  // Find override files
  const overridesDirPath = pathJoin(`${OVERRIDES_PATH}${dirPath.slice(PACKAGES_PATH.length)}`);
  let overrideFiles;
  try {
    overrideFiles = await readdir(overridesDirPath, { withFileTypes: true });
  } catch (err) {
    if (err?.code !== 'ENOENT') throw err;
  }

  if (overrideFiles) {
    for (const file of overrideFiles) {
      if (file.isDirectory()) continue;

      const filename = file.name;
      // `reason.txt` files are to document why override is used
      if (filename === 'reason.txt') continue;

      const ext = extname(filename),
        type = filename.slice(0, -ext.length),
        path = pathJoin(overridesDirPath, filename);

      assert(Object.hasOwn(overrides, type), `Unexpected override file: ${path}`);

      const originalPath = pathJoin(dirPath, filename);
      if (filenames[type]) {
        const originalFilename = filenames[type];
        assert(originalFilename === filename, `Unmatched override file: ${path} (original: ${originalFilename})`);
        await backupFile(originalPath);
      }

      filenames[type] = filename;
      overrides[type] = true;
      if (type === 'options') hasChangedOptions = true;

      await copyFile(path, originalPath);
    }
  }

  // Update options, save to file, and merge options with parent
  if (filenames.options) {
    const path = pathJoin(dirPath, filenames.options);
    const localOptions = JSON.parse(await readFile(path, 'utf8'));
    if (!overrides.options && updateOptions(localOptions)) {
      hasChangedOptions = true;
      await backupFile(path);
      await writeFile(path, JSON.stringify(localOptions, null, 2) + '\n');
    }
    options = mergeOptions(options, localOptions);
  }

  // Run Babel with updated options/input
  if (filenames.output && (hasChangedOptions || overrides.input) && !overrides.output) {
    const inputPath = pathJoin(dirPath, filenames.input),
      outputPath = pathJoin(dirPath, filenames.output);
    await backupFile(outputPath);
    await transform(inputPath, outputPath, options);
  }

  // Process subfolders
  for (const filename of dirFiles) {
    const path = pathJoin(dirPath, filename);
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

  let addExternalHelpersPlugin = true;
  if (Object.hasOwn(options, 'externalHelpers')) {
    if (!options.externalHelpers) addExternalHelpersPlugin = false;
    delete options.externalHelpers;
  }

  if (addExternalHelpersPlugin) {
    options.plugins.push(['@babel/plugin-external-helpers', { helperVersion: EXTERNAL_HELPERS_VERSION }]);
  }

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

/**
 * Backup file.
 * @param {string} path - Original path
 * @returns {undefined}
 */
async function backupFile(path) {
  const ext = extname(path),
    backupPath = `${path.slice(0, -ext.length)}.original${ext}`;
  await rename(path, backupPath);
}

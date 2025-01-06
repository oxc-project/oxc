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
import { extname, join as pathJoin } from 'path';

const PACKAGES = [
  'babel-plugin-transform-class-properties',
  'babel-plugin-transform-private-methods',
  'babel-plugin-transform-private-property-in-object',
  'babel-plugin-transform-logical-assignment-operators',
];
const FILTER_OUT_PRESETS = ['env'];
const FILTER_OUT_PLUGINS = [
  'transform-classes',
  'transform-block-scoping',
  'transform-destructuring',
];

const CLASS_PLUGINS = [
  'transform-class-properties',
  'transform-private-methods',
  'transform-private-property-in-object',
];

const PACKAGES_PATH = pathJoin(import.meta.dirname, '../coverage/babel/packages');

// These fixtures transform incorrectly by Babel. Haven't figured out why yet.
const IGNORED_FIXTURES = [
  'compile-to-class/constructor-collision-ignores-types',
  'compile-to-class/constructor-collision-ignores-types-loose',
];

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
  if (IGNORED_FIXTURES.some(p => dirPath.endsWith(p))) {
    return;
  }

  const files = await readdir(dirPath, { withFileTypes: true });

  const dirFiles = [],
    filenames = { options: null, input: null, output: null };

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

  // Update options, save to file, and merge options with parent
  if (filenames.options) {
    const path = pathJoin(dirPath, filenames.options);
    const localOptions = JSON.parse(await readFile(path, 'utf8'));
    if (updateOptions(localOptions)) {
      hasChangedOptions = true;
      await backupFile(path);
      await writeFile(path, JSON.stringify(localOptions, null, 2) + '\n');
    }
    options = { ...options, ...localOptions };
  }

  // Run Babel with updated options/input
  if (filenames.output && hasChangedOptions) {
    const inputPath = pathJoin(dirPath, filenames.input),
      outputPath = pathJoin(dirPath, filenames.output);

    const transformedCode = await transform(inputPath, options);
    const originalTransformedCode = await readFile(outputPath, 'utf8');

    if (transformedCode.trim() !== originalTransformedCode.trim()) {
      await backupFile(outputPath);
      await writeFile(outputPath, transformedCode);
    }
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
  if (ensureAllClassPluginsEnabled(options)) {
    hasChangedOptions = true;
  }

  return hasChangedOptions;
}

// Ensure all class plugins are enabled if any of class related plugins are enabled
function ensureAllClassPluginsEnabled(options) {
  let plugins = options.plugins;
  if (!plugins) return false;

  let already_enabled = [];
  let pluginOptions;
  plugins.forEach(plugin => {
    let pluginName = getName(plugin);
    if (CLASS_PLUGINS.includes(pluginName)) {
      if (Array.isArray(plugin) && plugin[1]) {
        // Store options for the plugin, so that we can ensure all plugins are
        // enabled with the same options
        pluginOptions = plugin[1];
      }
      already_enabled.push(pluginName);
    }
  });

  if (already_enabled.length) {
    CLASS_PLUGINS.forEach(pluginName => {
      if (!already_enabled.includes(pluginName)) {
        if (pluginOptions) {
          plugins.push([pluginName, pluginOptions]);
        } else {
          plugins.push(pluginName);
        }
      }
    });
    return true;
  } else {
    return false;
  }
}

/**
 * Transform input with Babel and save to output file.
 * @param {string} inputPath - Path of input file
 * @param {Object} options - Transform options
 * @returns {undefined}
 */
async function transform(inputPath, options) {
  options = {
    ...options,
    configFile: false,
    babelrc: false,
    cwd: import.meta.dirname,
  };
  delete options.BABEL_8_BREAKING;
  delete options.SKIP_babel7plugins_babel8core;
  delete options.minNodeVersion;
  delete options.validateLogs;
  delete options.SKIP_ON_PUBLISH;

  function prefixName(plugin, type) {
    if (Array.isArray(plugin)) {
      plugin = [...plugin];
      plugin[0] = `@babel/${type}-${plugin[0]}`;
    } else {
      plugin = `@babel/${type}-${plugin}`;
    }
    return plugin;
  }

  if (options.presets) {
    options.presets = options.presets.map((preset) => prefixName(preset, 'preset'));
  }

  options.plugins = (options.plugins || []).map((plugin) => prefixName(plugin, 'plugin'));

  let addExternalHelpersPlugin = true;
  if (Object.hasOwn(options, 'externalHelpers')) {
    if (!options.externalHelpers) addExternalHelpersPlugin = false;
    delete options.externalHelpers;
  }

  if (addExternalHelpersPlugin) {
    options.plugins.push([
      '@babel/plugin-external-helpers',
      { helperVersion: EXTERNAL_HELPERS_VERSION },
    ]);
  }

  const { code } = await transformFileAsync(inputPath, options);
  return code;
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

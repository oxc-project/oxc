"use strict";

const path = require("path");
const ignore = require("ignore");
const getFileContentOrNull = require("../utils/get-file-content-or-null");

/**
 * @param {string | null} ignorePath
 * @param {boolean | null} withNodeModules
 */
async function createIgnorer(ignorePath, withNodeModules) {
  const ignoreContent = ignorePath
    ? await getFileContentOrNull(path.resolve(ignorePath))
    : null;

  return _createIgnorer(ignoreContent, withNodeModules);
}

/**
 * @param {string | null} ignorePath
 * @param {boolean | null} withNodeModules
 */
createIgnorer.sync = function (ignorePath, withNodeModules) {
  const ignoreContent = !ignorePath
    ? null
    : getFileContentOrNull.sync(path.resolve(ignorePath));
  return _createIgnorer(ignoreContent, withNodeModules);
};

/**
 * @param {null | string} ignoreContent
 * @param {boolean | null} withNodeModules
 */
function _createIgnorer(ignoreContent, withNodeModules) {
  const ignorer = ignore().add(ignoreContent || "");
  if (!withNodeModules) {
    ignorer.add("node_modules");
  }
  return ignorer;
}

module.exports = createIgnorer;

// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/build-data.js
// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/utils-build-data.js

const fs = require("node:fs");
const path = require("node:path");
const envs = require("./compat-table/environments");
const parseEnvsVersions = require("./compat-table/build-utils/parse-envs-versions");
const interpolateAllResults = require("./compat-table/build-utils/interpolate-all-results");
const compareVersions = require("./compat-table/build-utils/compare-versions");
const { addElectronSupportFromChromium } = require("./chromium-to-electron");
const esFeatures = require(`./es-features`);
const customCompatData = require("./custom-compat-data");

const environments = [
  "chrome",
  "opera",
  "edge",
  "firefox",
  "safari",
  "node",
  "deno",
  "ie",
  "android",
  "ios",
  // 'phantom',
  "samsung",
  "rhino",
  "opera_mobile",
];

const envsVersions = parseEnvsVersions(envs);
// Override node versions
const nodeVersions = require("fs")
  .readFileSync("../coverage/node-compat-table/v8.versions")
  .toString()
  .replace(/v/g, "")
  .trim()
  .split("\n")
  .reverse()
  .map((version) => {
    const versions = version.split(".").map((n) => parseInt(n, 10));
    return {
      id: "node" + versions.join("_"),
      name: "node",
      version: versions,
    };
  });
const es5NodeVersions = new Set(["node0_4", "node0_6", "node0_8"]);
const es5EnvsVersions = envsVersions["node"].filter((n) => es5NodeVersions.has(n.id));
envsVersions["node"] = es5EnvsVersions.concat(nodeVersions);

const compatSources = ["es5", "es6", "es2016plus", "esnext"].map((source) => {
  const data = require(`./compat-table/data-${source}`);
  interpolateAllResults(data.tests, envs);
  return data;
});

const reformatNodeCompatTable = () => {
  const nodeCompatTableDir = path.join(__dirname, "../coverage/node-compat-table/results/v8");
  const testMap = {};
  const subtestMap = {};
  const tests = [];

  // Format the data like the kangax table
  for (const entry of fs.readdirSync(nodeCompatTableDir)) {
    // Note: this omits data for the "0.x.y" releases because the data isn't clean
    const match = /^([1-9]\d*\.\d+\.\d+)\.json$/.exec(entry);
    if (match) {
      const version = "node" + match[1].replace(/\./g, "_");
      const jsonPath = path.join(nodeCompatTableDir, entry);
      const json = JSON.parse(fs.readFileSync(jsonPath, "utf8"));

      for (const key in json) {
        if (key.startsWith("ES")) {
          const object = json[key];

          for (const key in object) {
            const testResult = object[key];
            const split = key.replace("<code>", "").replace("</code>", "").split("›");

            if (split.length === 2) {
              let test = testMap[split[1]];
              if (!test) {
                test = testMap[split[1]] = { name: split[1], res: {} };
                tests.push(test);
              }
              test.res[version] = testResult;
            } else if (split.length === 3) {
              const subtestKey = `${split[1]}: ${split[2]}`;
              let subtest = subtestMap[subtestKey];
              if (!subtest) {
                let test = testMap[split[1]];
                if (!test) {
                  test = testMap[split[1]] = { name: split[1], res: {} };
                  tests.push(test);
                }
                subtest = subtestMap[subtestKey] = { name: split[2], res: {} };
                test.subtests ||= [];
                test.subtests.push(subtest);
              }
              subtest.res[version] = testResult;
            }
          }
        }
      }
    }
  }

  return tests;
};

const flattenFeatureData = (tests) => {
  return tests.flatMap((test) => {
    if (!test.subtests) return test;
    return test.subtests.map((subtest) =>
      Object.assign({}, subtest, {
        name: test.name + " / " + subtest.name,
        group: test.name,
      }),
    );
  });
};

const compatibilityTests = compatSources.flatMap((data) => flattenFeatureData(data.tests));
const nodeTests = flattenFeatureData(reformatNodeCompatTable());

// Merge nodeTests into compatibilityTests
compatibilityTests.forEach((t) => {
  const found = nodeTests.find((test) => t.name == test.name);
  if (found) {
    // Remove all node data from compatibilityTests then merge nodeTests into it.
    Object.keys(t.res).forEach((key) => {
      if (key.startsWith("node")) {
        t.res[key] = undefined;
      }
    });
    Object.assign(t.res, found.res);
  } else {
    // Add missing node data.
    nodeVersions.forEach((v) => {
      t.res[v.id] = true;
    });
  }
});

const getLowestImplementedVersion = ({ features }, env, exclude = () => false) => {
  const tests = compatibilityTests.filter((test) => {
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
      if (res[id] !== true && res[id] !== "strict") {
        break;
      }
    }

    return envsVersions[env][i + 1];
  });

  if (envTests.length === 0 || envTests.some((t) => !t)) return null;

  const result = envTests.reduce((a, b) => {
    return compareVersions(a, b) > 0 ? a : b;
  });

  // NOTE(bng): A number of environments in compat-table changed to
  // include a trailing zero (node10 -> node10_0), so for now stripping
  // it to be consistent
  return result.version.join(".").replace(/\.0$/, "");
};

const expandFeatures = (features) =>
  features.flatMap((feat) => {
    if (feat.includes("/")) return [feat];
    return compatibilityTests
      .map((test) => test.name)
      .filter((name) => name === feat || name.startsWith(feat + " / "));
  });

const generateData = (environments, items) => {
  for (const item of items) {
    const targets = {};
    environments.forEach((env) => {
      const version = getLowestImplementedVersion(
        {
          features: expandFeatures(item.features),
        },
        env,
      );
      if (version) targets[env] = version;
    });
    addElectronSupportFromChromium(targets);

    item.targets = targets;
  }

  return items;
};

const items = generateData(environments, esFeatures);

// Merge custom compatibility data (for features not in compat-table)
const allItems = [...items, ...customCompatData];

fs.writeFileSync("./data.json", JSON.stringify(allItems, null, 2));

// https://github.com/babel/babel/blob/v7.26.2/packages/babel-compat-data/scripts/chromium-to-electron.js

const chromiumVersions = require('./chromium-versions');
const chromiumVersionList = Object.keys(chromiumVersions);

function chromiumToElectron(version) {
  if (chromiumVersions[version]) {
    return chromiumVersions[version];
  }
  const supportedVersion = chromiumVersionList.concat(version);
  supportedVersion.sort((a, b) => +a - +b);
  const nextSupportedVersion = supportedVersion[supportedVersion.indexOf(version) + 1];
  return chromiumVersions[nextSupportedVersion];
}

function addElectronSupportFromChromium(supportData) {
  if (supportData.chrome) {
    const electronVersion = chromiumToElectron(supportData.chrome);
    if (electronVersion) {
      supportData.electron = electronVersion;
    }
  }
}

module.exports.addElectronSupportFromChromium = addElectronSupportFromChromium;

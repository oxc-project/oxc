function loadModule(name, options) {
  return import(name, options);
}

function loadSource(name) {
  return import.source(name);
}

function loadDeferred(name) {
  return import.defer(name);
}

function loadWebpackChunk() {
  return import(/* webpackChunkName: "settings" */ './settings');
}

function loadViteUrl(url) {
  return import(/* @vite-ignore */ url);
}

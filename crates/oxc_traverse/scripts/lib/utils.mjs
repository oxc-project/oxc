/**
 * @param {string} name
 */
export function typeAndWrappers(name) {
  const wrappers = [];
  while (true) {
    const match = name.match(/^(.+?)<(.+)>$/);
    if (!match) break;
    wrappers.push(match[1]);
    name = match[2];
  }
  return { name, wrappers };
}

/**
 * @param {string} name
 */
export function camelToSnake(name) {
  let prefixLen = 1;
  for (const prefix of ['TS', 'JSX', 'JS']) {
    if (name.startsWith(prefix)) {
      prefixLen = prefix.length;
      break;
    }
  }
  return name.slice(0, prefixLen).toLowerCase() +
    name.slice(prefixLen).replace(/[A-Z]/g, c => `_${c.toLowerCase()}`);
}

/**
 * @param {string} name
 */
export function snakeToCamel(name) {
  let prefixLen = 0;
  for (const prefix of ['TS', 'JSX', 'JS']) {
    if (name.startsWith(`${prefix.toLowerCase()}_`)) {
      prefixLen = prefix.length + 1;
      break;
    }
  }
  return name.slice(0, prefixLen + 1).toUpperCase() +
    name.slice(prefixLen + 1).replace(/_([a-z])/g, (_, c) => c.toUpperCase());
}

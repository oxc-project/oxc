// Enable Codspeed plugin in CI only
const config = {};
if (process.env.CI) {
  const codspeedPlugin = (await import('@codspeed/vitest-plugin')).default;
  // @ts-ignore
  config.plugins = [codspeedPlugin()];
}

export default config;

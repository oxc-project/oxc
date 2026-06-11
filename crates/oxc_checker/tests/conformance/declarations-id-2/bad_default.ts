function buildConfig(): { mode: string } {
  return { mode: "fast" };
}

// Error: default-exported expression cannot be inferred.
export default buildConfig();

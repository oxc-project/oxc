{
  RESOLUTION_CACHE = new ExpiringCache(
    options.singleRun
      ? 'Infinity'
      : (options.cacheLifetime?.globLooooooooooooooooong ??
        DEFAULT_TSCONFIG_CACHE_DURATION_SECONDS),
  );
}

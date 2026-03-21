export function createStripeSchemaChecker(): void {
  createPluginModelSyncChecker({
    buildPartialDef: (container) => {
      const config = PluginUtils.configByKey(
        container.definition,
        pluginKey,
      ) as StripePluginDefinition | undefined;
    },
  });
}

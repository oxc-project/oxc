// @validateExhaustiveMemoizationDependencies
function Component() {
  useCallback(() => {
    const bar = () => {
      console.log(bar);
    };
  }, []);
}

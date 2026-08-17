function useMutate(document) {
  const updateData = useCallback(() => {
    document.data = {};
  }, [document]);
  const updateTitle = useCallback(() => {
    document.title = '';
  }, [document]);
  const updateIcon = useCallback(() => {
    document.icon = '';
  }, [document]);
  return [updateData, updateTitle, updateIcon];
}

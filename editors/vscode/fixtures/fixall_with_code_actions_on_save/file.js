export function processChanges(changeHistory, maxChangesToShow) {
  if (!changeHistory || !changeHistory.changes) {
      return [];
  }

  const { patch = [] } = changeHistory.changes;
  const summaryItems = [];
  const processedPaths = new Set();
  let totalChanges = 0;

  // Process patch operations
  patch.forEach(({ op, path, value }) => {
      totalChanges++;

      // Skip if we've reached the display limit
      if (summaryItems.length >= maxChangesToShow) return;

      // Format path for display
      const displayPath = formatPathForDisplay(path);

      // Skip duplicates or paths we've already processed
      if (processedPaths.has(displayPath)) return;
      processedPaths.add(displayPath);

      // Handle different operation types
      switch (op) {
          case 'add':
              // Special handling for references
              if (path.match(/^\/references\/\d+$/)) {
                  if (value && typeof value === 'object' && 'key' in value) {
                      summaryItems.push({
                          label: 'References.' + value.key,
                          displayValue: <span className='text-green-600 font-medium'>Added</span>
                      });
                  }
              } else {
                  summaryItems.push({
                      label: displayPath,
                      displayValue: <span className='text-green-600 font-medium'>Added</span>
                  });
              }
              break;

          case 'remove':
              summaryItems.push({
                  label: displayPath,
                  displayValue: <span className='text-red-600 font-medium'>Removed</span>
              });
              break;

          case 'replace':
              // Skip version changes as we display them separately
              if (path === '/version') return;

              summaryItems.push({
                  label: displayPath,
                  displayValue: <span className='text-blue-600 font-medium'>Updated</span>
              });
              break;

          default:
              summaryItems.push({
                  label: displayPath,
                  displayValue: <span className='text-gray-600'>{op}</span>
              });
      }
  });

  // Add metadata about more changes
  summaryItems.moreChangesCount = Math.max(0, totalChanges - maxChangesToShow);

  return summaryItems;
}

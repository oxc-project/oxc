// A private function name can escape through callbacks or JSX without a local declaration.
export const ExportedName = function Node() {
  return <Tree renderItem={() => <Node />} />;
};

export const Nested = function Node({ value }) {
  return <Tree renderItem={() => () => <Node value={value} />} />;
};

export const Direct = function Node({ depth }) {
  return depth > 0 ? <Node depth={depth - 1} /> : null;
};

export const Shadowed = function Node({ Node }) {
  return <Tree renderItem={() => <Node />} />;
};

export const useExported = function useRecursive() {
  "use memo";
  return useRecursive;
};

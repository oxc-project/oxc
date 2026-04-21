// https://github.com/oxc-project/oxc/issues/17927
// https://github.com/typescript-eslint/typescript-eslint/issues/10746
// When a type parameter shadows a namespace, qualified name references
// should still resolve to the namespace (not the type parameter).

namespace Database {
  export type Table<T> = T;
}

// Database.Table should reference the namespace Database, not the type parameter
type Test<Database> = Database.Table<Database>;

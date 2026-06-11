export type Pair<A, B> = { first: A; second: B };

export type Boxed<T> = { value: T };

export type WithDefault<T = string> = { item: T };

export type KeyHolder<K extends string> = { key: K };

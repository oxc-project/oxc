export interface Container<T> {
  content: T;
  replace(next: T): void;
}

export interface Lookup<K extends string, V = number> {
  key: K;
  value: V;
}

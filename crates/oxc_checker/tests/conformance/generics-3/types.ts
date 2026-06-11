export type Result<T, E = string> =
  | { ok: true; value: T }
  | { ok: false; error: E };

export interface Repository<Id extends string | number, Entity> {
  find(id: Id): Entity | null;
  save(entity: Entity): Id;
}

export type Nested<T> = { inner: Result<T> };

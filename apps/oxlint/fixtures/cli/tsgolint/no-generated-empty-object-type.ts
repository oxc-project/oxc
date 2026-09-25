type Data = { name: string; value: number };
// These operations resolve to the empty object type.
type EmptyOmit = Omit<Data, "name" | "value">;
type EmptyPick = Pick<Data, never>;
type EmptyNonNullable = NonNullable<unknown>;
type EmptyIntersection = Omit<Data, "name" | "value"> & Omit<Data, "name" | "value">;
// These operations retain properties.
type WithValue = Omit<Data, "name">;
type WithOther = Omit<Data, "name" | "value"> & { other: string };

type Keys<T> = T extends infer U ? keyof U : never;
type Mapped<T extends object> = { [Key in Keys<T>]: Key };
type Referenced<T extends object> = Mapped<T>;

export {};

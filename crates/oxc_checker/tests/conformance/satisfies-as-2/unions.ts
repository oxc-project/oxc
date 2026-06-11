export type Shape =
  | { kind: "circle"; radius: number }
  | { kind: "square"; side: number };

export type Id = string | number;

export type Status = "open" | "closed";

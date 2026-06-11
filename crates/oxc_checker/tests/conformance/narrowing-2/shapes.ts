export interface Circle {
  kind: "circle";
  radius: number;
}

export interface Square {
  kind: "square";
  side: number;
}

export interface Triangle {
  kind: "triangle";
  base: number;
  height: number;
}

export type Shape = Circle | Square | Triangle;

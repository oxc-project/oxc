// Interface targets, including an interface extending another interface.
export interface Shape {
  kind: string;
  area: number;
}

export interface Circle extends Shape {
  radius: number;
}

export const ok_shape: Shape = { kind: "square", area: 4 };

export const ok_circle: Circle = { kind: "circle", area: 3.14, radius: 1 };

export const bad_circle_missing_inherited: Circle = { kind: "circle", radius: 1 };

export const bad_circle_missing_own: Circle = { kind: "circle", area: 3.14 };

// Derived interface value assignable to its base: silent.
export const ok_circle_as_shape: Shape = ok_circle;

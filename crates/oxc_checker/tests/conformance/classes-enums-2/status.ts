export enum Status {
  Active = 1,
  Inactive = 2,
  Pending = 3,
}

export const enum Direction {
  Up = "UP",
  Down = "DOWN",
}

export enum Level {
  Low,
  Mid,
  High,
}

const ok_local_status_number: number = Status.Active;
const ok_local_direction_string: string = Direction.Up;

import { Direction, Level, Status } from "./status";

const ok_status: Status = Status.Active;
const ok_status_as_number: number = Status.Inactive;
const ok_member_type: Status.Pending = Status.Pending;
const ok_direction_as_string: string = Direction.Up;
const ok_const_enum_value: Direction = Direction.Down;

function wants_number(count: number): number {
  return count;
}

const bad_arbitrary_number: Status = 99;
const bad_direction_literal: Direction = "UP";
const bad_member_mismatch: Status.Active = Status.Inactive;
const ok_roundtrip: number = wants_number(Status.Pending);

function describe_level(level: Level): string {
  if (level === Level.High) {
    return "high";
  }
  return "other";
}

const ok_described: string = describe_level(Level.Mid);
const bad_cross_enum_arg: string = describe_level(Status.Active);

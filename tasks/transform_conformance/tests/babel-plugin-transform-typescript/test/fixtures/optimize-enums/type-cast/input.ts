enum Direction {
  Up,
  Down,
}

Direction.Up as Direction;
Direction["Down"] as Direction;
Direction.Up satisfies Direction;
Direction.Down!;
<Direction>Direction.Up;

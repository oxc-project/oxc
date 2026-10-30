enum Direction {
  Up = 1,
  Down = 2,
}

const enum Status {
  Active = "active",
}

export function App() {
  return (
    <div>
      {Direction.Up}:{Status.Active}
    </div>
  );
}

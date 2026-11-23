import { useState } from "react";

function Component({ value }) {
  const [state, setState] = useState(0);
  enum Value {
    Current = value,
  }
  setState(1);
  return (
    <div>
      {Value.Current}
      {state}
    </div>
  );
}

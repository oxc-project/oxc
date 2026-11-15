import { useState } from "react";

function Component(value = <span />) {
  const [state, setState] = useState(0);
  setState(1);
  return (
    <div>
      {value}
      {state}
    </div>
  );
}

import { useEffect } from "react";

function Component() {
  useEffect(() => {
    let i = 0;
    for (; i < 3; i++) {
      console.log(i);
    }
  });
  return null;
}

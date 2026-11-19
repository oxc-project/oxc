// @validateNoCapitalizedCalls @outputMode:"lint" @panicThreshold:"none"
// ALL-CAPS methods are constants, not components.
export function AllCapsMethod() {
  const now = new Date();
  const startDate = new Date(Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), 1));
  return <div>{startDate.toString()}</div>;
}

// A capitalized method is still called like a component.
export function CapitalizedMethod({ obj }) {
  return <div>{obj.Child()}</div>;
}

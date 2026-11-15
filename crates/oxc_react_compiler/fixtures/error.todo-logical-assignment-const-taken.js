// @panicThreshold:"none"
function NullishComponent() {
  const value = null;
  value ??= 2;
  return value;
}

function OrComponent() {
  const value = false;
  value ||= true;
  return value;
}

function AndComponent() {
  const value = true;
  value &&= false;
  return value;
}

// @panicThreshold:"none"
const moduleValue = 1;

function NullishComponent() {
  const value = 1;
  value ??= 2;
  return value;
}

function OrComponent() {
  const value = true;
  value ||= false;
  return value;
}

function AndComponent() {
  const value = false;
  value &&= true;
  return value;
}

function CapturedComponent() {
  const value = 1;
  const read = () => value;
  value ??= 2;
  return read();
}

function ModuleComponent() {
  moduleValue ??= 2;
  return moduleValue;
}

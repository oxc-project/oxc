export function First() {
  useState(0);
  return null;
}

export function Second() {
  // @refresh reset
  useEffect(() => {});
  return null;
}

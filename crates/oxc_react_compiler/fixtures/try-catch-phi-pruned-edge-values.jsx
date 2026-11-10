export function Component({value}) {
  let result = 'initial';
  try {
    const prefix = 'p';
    result = read(prefix, value);
  } catch {
    result = 'caught';
  }
  return <div>{result}</div>;
}

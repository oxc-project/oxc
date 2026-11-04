function Component({suffix, offset}: {suffix: string | null; offset: number | null}) {
  let text = 'prefix';
  let count = 10;

  if (suffix) {
    text += suffix;
  }
  if (offset) {
    count -= offset;
  }

  return <div>{text}: {count}</div>;
}

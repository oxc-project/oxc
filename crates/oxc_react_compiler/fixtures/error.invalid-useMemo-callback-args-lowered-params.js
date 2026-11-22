function Component({a}) {
  const x = useMemo((n = 1) => n + a, [a]);
  const y = useMemo(({k}) => k + a, [a]);
  return x + y;
}

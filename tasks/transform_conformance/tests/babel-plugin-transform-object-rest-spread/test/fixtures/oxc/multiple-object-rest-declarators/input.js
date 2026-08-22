function merge(e, t) {
  const { ariaAttributes: s, style: r, ...n } = e,
    { ariaAttributes: i, style: l, ...f } = t;
  return [s, r, n, i, l, f];
}

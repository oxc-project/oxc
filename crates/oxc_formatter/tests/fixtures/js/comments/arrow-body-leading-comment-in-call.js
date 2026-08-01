pinia._p.push(() =>
  // @ts-expect-error: invalid state
  ({ external: { dark: true } })
);

call(() =>
  // comment
  ({})
);

call(() =>
  // comment
  []
);

call(() =>
  /* block comment */
  ({})
);

call(first, () =>
  // comment
  ({})
);

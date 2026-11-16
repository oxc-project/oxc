a ? <T,>(b: B) => (c) : d => e;
a ? async <T extends U>(b) => (c) : d => e;
a ? <T = U>(b) => (c) : d => e;
a ? async <const T,>(b) => (c) : d => e;
a ? async <T,>(b): R => () => (c) : d => e;
a ? async <T, U>(b): R => () => (c) : d => e;
a ? async <T,>(): R => () => (c) : d => e;
a ? async <T,>(...b): R => () => (c) : d => e;

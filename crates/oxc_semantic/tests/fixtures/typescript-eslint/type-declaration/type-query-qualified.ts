const x = { y: { z: 1 } };
type T = typeof x.y.z;
type Unresolved = x.y.z;

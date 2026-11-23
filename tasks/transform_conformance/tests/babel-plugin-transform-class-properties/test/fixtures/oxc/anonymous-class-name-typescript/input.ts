const A = class { static y = 1 } as any;
const B = class { static y = 1 } satisfies object;
const C = class { static y = 1 }!;
const D = <any>class { static y = 1 };
const E = (class { static y = 1 } as any);

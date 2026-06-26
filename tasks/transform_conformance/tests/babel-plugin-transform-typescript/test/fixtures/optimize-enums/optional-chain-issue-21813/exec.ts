// https://github.com/oxc-project/oxc/issues/21813
enum c_num { x = 123 }
enum d_num { x = 123 }

expect([c_num?.x, d_num?.['x']]).toEqual([123, 123]);

let first, second, rest;
[...{ ...rest }] = [1, 2];
[first, ...{ 0: second, ...rest }] = [1, 2, 3];
[...[{ ...rest }]] = [{ value: 1 }];
for ([...{ ...rest }] of [[1, 2]]) {}
for ([...{ ...rest }] in { ab: 0 }) {}

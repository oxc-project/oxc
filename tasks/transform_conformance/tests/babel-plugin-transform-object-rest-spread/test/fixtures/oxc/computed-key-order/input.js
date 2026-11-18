let value, rest;
({ [key()]: value, ...rest } = source());
[...{ [key()]: value, ...rest }] = iterable();
condition && ({ [key()]: value, ...rest } = source());
const result = ({ [key()]: value, ...rest } = source());

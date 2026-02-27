// `<<` is disambiguated: speculatively tried as `<` for type args, fails, rewinds to `<<`
const a = n << 2;

// Successful type argument parsing with `<` and `>`
const b = id<number>(42);

// `>` after type args is disambiguated: speculatively tried as end of type args, fails,
// rewinds to binary expression `n < (1 >> (0))`
const c = n<1>>(0);

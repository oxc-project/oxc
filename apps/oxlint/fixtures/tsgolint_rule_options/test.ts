type Day = 'Monday' | 'Tuesday';

declare const day: Day;
let result = 0;

// This should NOT error when considerDefaultExhaustiveForUnions is true
// because the default case makes it exhaustive
switch (day) {
  case 'Monday':
    result = 1;
    break;
  default:
    result = 3;
    break;
}

export { result };

// Examples of incorrect code for require-array-sort-compare rule

const numbers = [3, 1, 4, 1, 5];
numbers.sort(); // Lexicographic sort, not numeric

const mixedArray = ['10', '2', '1'];
mixedArray.sort(); // Might be intended, but explicit compareFn is clearer

[3, 1, 4].sort(); // Will sort as strings: ['1', '3', '4']
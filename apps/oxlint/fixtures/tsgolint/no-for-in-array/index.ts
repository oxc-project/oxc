// Examples of incorrect code for no-for-in-array rule

const arr = [1, 2, 3];

for (const i in arr) {
  console.log(arr[i]);
}

for (const i in arr) {
  console.log(i, arr[i]);
}

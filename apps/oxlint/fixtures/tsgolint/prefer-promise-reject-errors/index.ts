// Examples of incorrect code for prefer-promise-reject-errors rule

Promise.reject('error'); // rejecting with string

Promise.reject(42); // rejecting with number

Promise.reject(true); // rejecting with boolean

Promise.reject({ message: 'error' }); // rejecting with plain object

Promise.reject(null); // rejecting with null
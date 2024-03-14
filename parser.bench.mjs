import {bench, describe} from 'vitest';

function fibonacci(n) {
    if (n < 2) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

describe('fibo', () => {
    bench('fibo 10', () => {
        fibonacci(10);
    });
    bench('fibo 15', () => {
        fibonacci(15);
    });
});

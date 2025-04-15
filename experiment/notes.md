# Benchmarks

## Main thread benchmark

```
1 iterations in main thread
Without walk.
Total time: 100.632583 ms
Time per iteration: 100.632583 ms
Iteration speed: 9.937139345812081 Hz
----------
8 iterations in main thread
Without walk.
Total time: 549.8210839999999 ms
Time per iteration: 68.72763549999999 ms
Iteration speed: 14.550187747983854 Hz
----------
64 iterations in main thread
Without walk.
Total time: 3862.7563339999997 ms
Time per iteration: 60.355567718749995 ms
Iteration speed: 16.568479724354262 Hz
----------
64 iterations in main thread
With walk.
Total time: 5140.610916999999 ms
Time per iteration: 80.32204557812499 ms
Iteration speed: 12.449882131392597 Hz
```

## Worker threads benchmark

```
1 iterations in 1 threads ( 1 iterations per thread )
Without walk.
Total time: 138.33975 ms
Time per iteration: 138.33975 ms
Iteration speed: 7.228580361031446 Hz
----------
8 iterations in 8 threads ( 1 iterations per thread )
Without walk.
Total time: 405.13320899999997 ms
Time per iteration: 405.13320899999997 ms
Iteration speed: 2.4683239432983637 Hz
----------
256 iterations in 4 threads ( 64 iterations per thread )
Without walk.
Total time: 6034.483166 ms
Time per iteration: 94.28879946875 ms
Iteration speed: 10.605713569737715 Hz
----------
384 iterations in 6 threads ( 64 iterations per thread )
Without walk.
Total time: 7392.1475 ms
Time per iteration: 115.5023046875 ms
Iteration speed: 8.657835899513639 Hz
----------
512 iterations in 8 threads ( 64 iterations per thread )
Without walk.
Total time: 9478.806292 ms
Time per iteration: 148.1063483125 ms
Iteration speed: 6.751905042517352 Hz
----------
512 iterations in 8 threads ( 64 iterations per thread )
With walk.
Total time: 11186.226208 ms
Time per iteration: 174.7847845 ms
Iteration speed: 5.7213218121969875 Hz
```

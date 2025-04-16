# Benchmarks

## Main thread benchmark

```
1 iterations in main thread
Without walk
Total time: 107.30825 ms
Time per iteration: 107.30825 ms
Iteration speed: 9.31894798396209 Hz
----------
8 iterations in main thread
Without walk
Total time: 541.6411250000001 ms
Time per iteration: 67.70514062500001 ms
Iteration speed: 14.769927228106985 Hz
----------
64 iterations in main thread
Without walk
Total time: 3809.367084 ms
Time per iteration: 59.5213606875 ms
Iteration speed: 16.800691187995785 Hz
----------
64 iterations in main thread
With walk
Total time: 5052.234334000001 ms
Time per iteration: 78.94116146875001 ms
Iteration speed: 12.66766261598348 Hz
----------
1 iterations in main thread
Without deserialization
Total time: 17.66033299999981 ms
Time per iteration: 17.66033299999981 ms
Iteration speed: 56.62407384957072 Hz
----------
8 iterations in main thread
Without deserialization
Total time: 139.08891699999913 ms
Time per iteration: 17.38611462499989 ms
Iteration speed: 57.51716364288069 Hz
----------
64 iterations in main thread
Without deserialization
Total time: 1101.0636670000004 ms
Time per iteration: 17.204119796875005 ms
Iteration speed: 58.125612458339326 Hz
```

## Worker threads benchmark

```
1 iterations in 1 threads ( 1 iterations per thread )
Without walk
Total time: 139.33325 ms
Time per iteration: 139.33325 ms
Iteration speed: 7.177037785309681 Hz
----------
8 iterations in 8 threads ( 1 iterations per thread )
Without walk
Total time: 448.37999999999994 ms
Time per iteration: 448.37999999999994 ms
Iteration speed: 2.230251126276819 Hz
----------
256 iterations in 4 threads ( 64 iterations per thread )
Without walk
Total time: 5889.267083000001 ms
Time per iteration: 92.01979817187501 ms
Iteration speed: 10.867226617169877 Hz
----------
384 iterations in 6 threads ( 64 iterations per thread )
Without walk
Total time: 7479.010583 ms
Time per iteration: 116.859540359375 ms
Iteration speed: 8.557281647050184 Hz
----------
512 iterations in 8 threads ( 64 iterations per thread )
Without walk
Total time: 9465.908333000001 ms
Time per iteration: 147.90481770312502 ms
Iteration speed: 6.761104983119637 Hz
----------
768 iterations in 12 threads ( 64 iterations per thread )
Without walk
Total time: 14449.624166000001 ms
Time per iteration: 225.77537759375002 ms
Iteration speed: 4.429180943722547 Hz
----------
512 iterations in 8 threads ( 64 iterations per thread )
With walk
Total time: 10916.172417000002 ms
Time per iteration: 170.56519401562502 ms
Iteration speed: 5.86286085957486 Hz
----------
256 iterations in 1 threads ( 256 iterations per thread )
Without deserialization
Total time: 4760.252750000007 ms
Time per iteration: 18.594737304687527 ms
Iteration speed: 53.77865702614207 Hz
----------
1024 iterations in 4 threads ( 256 iterations per thread )
Without deserialization
Total time: 4966.5533330000035 ms
Time per iteration: 19.400598957031264 ms
Iteration speed: 51.54480035460838 Hz
----------
1536 iterations in 6 threads ( 256 iterations per thread )
Without deserialization
Total time: 5172.1603749999995 ms
Time per iteration: 20.203751464843748 ms
Iteration speed: 49.495758336766585 Hz
----------
2048 iterations in 8 threads ( 256 iterations per thread )
Without deserialization
Total time: 5411.273958000005 ms
Time per iteration: 21.13778889843752 ms
Iteration speed: 47.30863785255793 Hz
----------
3072 iterations in 12 threads ( 256 iterations per thread )
Without deserialization
Total time: 6437.601500000004 ms
Time per iteration: 25.146880859375017 ms
Iteration speed: 39.766363295398115 Hz
```

Notes:

- With deserialization, performance drops off after 4 threads.
- Without deserialization, performance scales with thread count up to 8 threads.
- Possibly this indicates garbage collection is the slow-down.

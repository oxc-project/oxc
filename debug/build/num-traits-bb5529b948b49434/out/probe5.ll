; ModuleID = 'probe5.f80dab502082319b-cgu.0'
source_filename = "probe5.f80dab502082319b-cgu.0"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"
target triple = "x86_64-apple-macosx10.7.0"

@alloc_8efb7e84327a9b774899de722dfe29ca = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/ad963232d9b987d66a6f8e6ec4141f672b8b9900/library/core/src/num/mod.rs" }>, align 1
@alloc_7bbc4bdde9e75bfdf73c67bb3da9ecb9 = private unnamed_addr constant <{ ptr, [16 x i8] }> <{ ptr @alloc_8efb7e84327a9b774899de722dfe29ca, [16 x i8] c"K\00\00\00\00\00\00\00w\04\00\00\05\00\00\00" }>, align 8
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe5::probe
; Function Attrs: uwtable
define void @_ZN6probe55probe17h4ffc0a410e63563dE() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false)
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17hd7c6812d35bd767cE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17ha85fdbcca3e90be3E(ptr align 1 @str.0, i64 25, ptr align 8 @alloc_7bbc4bdde9e75bfdf73c67bb3da9ecb9) #3
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17hd7c6812d35bd767cE.exit": ; preds = %start
  ret void
}

; Function Attrs: nocallback nofree nosync nounwind willreturn memory(none)
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn uwtable
declare void @_ZN4core9panicking5panic17ha85fdbcca3e90be3E(ptr align 1, i64, ptr align 8) unnamed_addr #2

attributes #0 = { uwtable "frame-pointer"="all" "probe-stack"="inline-asm" "target-cpu"="core2" }
attributes #1 = { nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { cold noinline noreturn uwtable "frame-pointer"="all" "probe-stack"="inline-asm" "target-cpu"="core2" }
attributes #3 = { noreturn }

!llvm.module.flags = !{!0}

!0 = !{i32 8, !"PIC Level", i32 2}

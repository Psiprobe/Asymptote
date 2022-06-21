; ModuleID = 'probe6.3d6d369e-cgu.0'
source_filename = "probe6.3d6d369e-cgu.0"
target datalayout = "e-m:e-p:32:32-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

%"core::panic::location::Location" = type { { [0 x i8]*, i32 }, i32, i32 }

@alloc1 = private unnamed_addr constant <{ [75 x i8] }> <{ [75 x i8] c"/rustc/db9d1b20bba1968c1ec1fc49616d4742c1725b4b/library/core/src/num/mod.rs" }>, align 1
@alloc2 = private unnamed_addr constant <{ i8*, [12 x i8] }> <{ i8* getelementptr inbounds (<{ [75 x i8] }>, <{ [75 x i8] }>* @alloc1, i32 0, i32 0, i32 0), [12 x i8] c"K\00\00\007\03\00\00\05\00\00\00" }>, align 4
@str.0 = internal constant [25 x i8] c"attempt to divide by zero"

; probe6::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe65probe17h05c2bc642e4ecbefE() unnamed_addr #0 {
start:
  %0 = call i1 @llvm.expect.i1(i1 false, i1 false) #3
  br i1 %0, label %panic.i, label %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h4413b192f6156a0bE.exit"

panic.i:                                          ; preds = %start
; call core::panicking::panic
  call void @_ZN4core9panicking5panic17h35778e3c18266c93E([0 x i8]* nonnull align 1 bitcast ([25 x i8]* @str.0 to [0 x i8]*), i32 25, %"core::panic::location::Location"* align 4 dereferenceable(16) bitcast (<{ i8*, [12 x i8] }>* @alloc2 to %"core::panic::location::Location"*)) #4
  unreachable

"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h4413b192f6156a0bE.exit": ; preds = %start
  br label %bb1

bb1:                                              ; preds = %"_ZN4core3num21_$LT$impl$u20$u32$GT$10div_euclid17h4413b192f6156a0bE.exit"
  ret void
}

; Function Attrs: nofree nosync nounwind readnone willreturn
declare i1 @llvm.expect.i1(i1, i1) #1

; core::panicking::panic
; Function Attrs: cold noinline noreturn nounwind
declare dso_local void @_ZN4core9panicking5panic17h35778e3c18266c93E([0 x i8]* nonnull align 1, i32, %"core::panic::location::Location"* align 4 dereferenceable(16)) unnamed_addr #2

attributes #0 = { nounwind "target-cpu"="generic" }
attributes #1 = { nofree nosync nounwind readnone willreturn }
attributes #2 = { cold noinline noreturn nounwind "target-cpu"="generic" }
attributes #3 = { nounwind }
attributes #4 = { noreturn nounwind }

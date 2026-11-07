//! [`Vec128`], a 128-bit SIMD vector which is generic over the CPU type.
//!
//! The methods defined here are the ones which every CPU type has.
//! Each method chooses its implementation depending on the CPU type, using [`cpu_select!`].
//!
//! [`avx2`] and [`neon`] modules add further methods which are exclusive to specific
//! CPU feature sets, e.g. [`movemask`] on x86_64.
//!
//! On x86_64, these are 128-bit operations, so the intrinsics are from SSE2 to SSE4.1,
//! which every CPU with AVX2 supports.
//!
//! The `!`, `&`, `|` and `^` operators for [`Vec128`] are implemented here too.
//!
//! [`avx2`]: super::avx2
//! [`neon`]: super::neon
//! [`movemask`]: Vec128::movemask

// These methods must be inlined to get the `target_feature`s of the functions calling them,
// otherwise the intrinsics also can't be inlined into them - 2 function calls for each method
#![expect(clippy::inline_always)]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::{
    CpuSimd, VecData128,
    arrays::{IntArray64, IntArray128},
    macros::{cpu_select, unused},
    operand::Operand128,
};

/// 128-bit SIMD vector.
///
/// Holds a `C: CpuSimd`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec128`] is the same size as the type it wraps.
#[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
#[derive(Clone, Copy)]
pub struct Vec128<C: CpuSimd> {
    inner: Vec128Inner,
    cpu: C,
}

/// The platform's SIMD type, which [`Vec128`] methods work with.
///
/// [`__m128i`] on x86_64, and [`uint8x16_t`] on aarch64.
///
/// It doesn't depend on the CPU type, because only one CPU type's vectors can exist
/// on each platform. See [`CpuSimd`].
///
/// On other platforms, a [`Vec128`] can never be created, and this type is a [`VecData128`],
/// which has the same size and alignment.
///
/// Both [`__m128i`] and [`uint8x16_t`] are 16 bytes, aligned on 16, so a [`Vec128`]
/// can store this type directly, and its layout is the same on every platform.
/// [`Vec256`]'s separate storage type is not required here.
///
/// [`__m128i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m128i.html
/// [`uint8x16_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16_t.html
/// [`Vec256`]: super::Vec256
type Vec128Inner = cfg_select! {
    target_arch = "x86_64" => __m128i,
    target_arch = "aarch64" => uint8x16_t,
    _ => VecData128,
};

// Methods which every CPU type has.
// A method which needs different code for different CPU types chooses it with `cpu_select!`.

/// Internal methods.
impl<C: CpuSimd> Vec128<C> {
    /// Create a [`Vec128`] wrapping `inner`.
    ///
    /// Internal method only.
    /// `pub(super)` so that `Vec256` methods and the `avx2` and `neon` modules can use it.
    #[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub(super) fn new(inner: Vec128Inner, cpu: C) -> Self {
        Self { inner, cpu }
    }

    /// Get the platform's SIMD type which this [`Vec128`] wraps.
    ///
    /// Internal method only.
    /// `pub(super)` so that `Vec256` methods and the `avx2` and `neon` modules can use it.
    #[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub(super) fn inner(self) -> Vec128Inner {
        self.inner
    }
}

/// Accessors.
impl<C: CpuSimd> Vec128<C> {
    /// Get the [`CpuSimd`] token from this [`Vec128`].
    #[inline(always)]
    pub fn cpu(self) -> C {
        self.cpu
    }
}

/// Creation methods.
impl<C: CpuSimd> Vec128<C> {
    /// Create a [`Vec128`] with all bits zero.
    #[cfg_attr(not(any(test, target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub fn zero(cpu: C) -> Self {
        Self::splat(0, cpu)
    }

    /// Create a [`Vec128`] by repeating `n` 16 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_set1_epi8(n.cast_signed()) }, cpu)
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `cpu` is a `Neon`, so CPU supports Neon
                Self::new(unsafe { vdupq_n_u8(n) }, cpu)
            }
            _ => unused!(n, cpu)
        })
    }

    /// Create a [`Vec128`] by repeating `n` 4 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_set1_epi32(n.cast_signed()) }, cpu)
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `cpu` is a `Neon`, so CPU supports Neon
                Self::new(unsafe { vreinterpretq_u8_u32(vdupq_n_u32(n)) }, cpu)
            }
            _ => unused!(n, cpu)
        })
    }
}

/// Loads and stores.
impl<C: CpuSimd> Vec128<C> {
    /// Read a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must be valid for reading 16 bytes.
    /// - `ptr` must be aligned on 16.
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes, and aligned.
                // `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2,
                // which includes SSE2.
                Self::new(unsafe { _mm_load_si128(ptr) }, cpu)
            }
            _ => {
                // Neon has no separate aligned load instruction,
                // so this is the same as `load_unaligned`.
                // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
                unsafe { Self::load_unaligned(ptr, cpu) }
            }
        })
    }

    /// Read a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 16 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
                // `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2,
                // which includes SSE2.
                Self::new(unsafe { _mm_loadu_si128(ptr) }, cpu)
            }
            Neon => {
                // SAFETY: Caller guarantees `ptr` is valid for reading 16 bytes.
                // `Neon` arm only runs if `cpu` is a `Neon`, so CPU supports Neon.
                Self::new(unsafe { vld1q_u8(ptr) }, cpu)
            }
            _ => unused!(ptr, cpu)
        })
    }

    /// Read 8 bytes at pointer `ptr` into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 8 bytes.
    #[inline(always)]
    pub unsafe fn load_low_unaligned(ptr: *const u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
                // `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2,
                // which includes SSE2.
                Self::new(unsafe { _mm_loadl_epi64(ptr) }, cpu)
            }
            Neon => {
                // SAFETY: Caller guarantees `ptr` is valid for reading 8 bytes.
                // `Neon` arm only runs if `cpu` is a `Neon`, so CPU supports Neon.
                Self::new(unsafe { vcombine_u8(vld1_u8(ptr), vdup_n_u8(0)) }, cpu)
            }
            _ => unused!(ptr, cpu)
        })
    }

    /// Read a [`Vec128`] from a [`VecData128`], which is aligned on 16.
    #[inline(always)]
    pub fn read(data: &VecData128, cpu: C) -> Self {
        // SAFETY: `VecData128` guarantees `data.as_ptr()` is valid for aligned reads of 16 bytes
        unsafe { Self::load(data.as_ptr(), cpu) }
    }

    /// Read a [`Vec128`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 16 bytes in total
    /// (e.g. `&[u8; 16]`, `&[i64; 2]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray128, cpu: C) -> Self {
        // SAFETY: `IntArray128` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 16 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }

    /// Read a slice of integers into the low half of a [`Vec128`],
    /// with no alignment requirement. The high half is set to zero.
    ///
    /// `slice` can be any slice of integers which is 8 bytes in total
    /// (e.g. `&[u8; 8]`, `&[i32; 2]`).
    #[inline(always)]
    pub fn read_low_unaligned(slice: &impl IntArray64, cpu: C) -> Self {
        // SAFETY: `IntArray64` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 8 bytes
        unsafe { Self::load_low_unaligned(slice.as_ptr(), cpu) }
    }

    /// Store a [`Vec128`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must be valid for writing 16 bytes.
    /// - `ptr` must be aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes, and aligned.
                // `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2.
                unsafe { _mm_store_si128(ptr, self.inner()) }
            }
            _ => {
                // Neon has no separate aligned store instruction,
                // so this is the same as `store_unaligned`.
                // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
                unsafe { self.store_unaligned(ptr) }
            }
        });
    }

    /// Store a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 16 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
                // `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2.
                unsafe { _mm_storeu_si128(ptr, self.inner()) }
            }
            Neon => {
                // This is all `vst1q_u8` does. Calling it would give the same instructions,
                // but it's a function which the compiler has to instantiate and then inline.
                // SAFETY: Caller guarantees `ptr` is valid for writing 16 bytes.
                unsafe { ptr.cast::<uint8x16_t>().write_unaligned(self.inner()) }
            }
            _ => unused!(self, ptr)
        });
    }

    /// Store the low 8 bytes of a [`Vec128`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 8 bytes.
    #[inline(always)]
    pub unsafe fn store_low_unaligned(self, ptr: *mut u8) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m128i>();

                // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
                // `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2.
                unsafe { _mm_storel_epi64(ptr, self.inner()) }
            }
            Neon => {
                // SAFETY: Caller guarantees `ptr` is valid for writing 8 bytes.
                // `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon.
                unsafe { vst1_u8(ptr, vget_low_u8(self.inner())) }
            }
            _ => unused!(self, ptr)
        });
    }

    /// Write a [`Vec128`] into a [`VecData128`], which is aligned on 16.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn write(self, dst: &mut VecData128) {
        // SAFETY: `VecData128` guarantees `dst.as_mut_ptr()`
        // is valid for aligned writes of 16 bytes
        unsafe { self.store(dst.as_mut_ptr()) };
    }
}

/// Bitwise ops.
impl<C: CpuSimd> Vec128<C> {
    /// Check if all the bits of this [`Vec128`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE4.1
                unsafe { _mm_testz_si128(self.inner(), self.inner()) != 0 }
            }
            _ => {
                // Neon has no equivalent of x86's `ptest`, so combine the 2 halves,
                // and test the result for 0
                (self.get_u64::<0>() | self.get_u64::<1>()) == 0
            }
        })
    }

    /// Invert all the bits of this [`Vec128`].
    #[inline(always)]
    pub fn not(self) -> Self {
        self.xor(Self::splat(0xFF, self.cpu()))
    }

    /// AND this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_and_si128(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vandq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// OR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_or_si128(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vorrq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// XOR this [`Vec128`] with either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_xor_si128(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { veorq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Shift each of the 8 `u16`s of this [`Vec128`] left by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    pub fn shl_u16<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_slli_epi16::<BITS>(self.inner()) }, self.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let shifted = vshlq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner()));
                    vreinterpretq_u8_u16(shifted)
                };
                Self::new(inner, self.cpu())
            }
            _ => unused!(self)
        })
    }

    /// Shift each of the 8 `u16`s of this [`Vec128`] right by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    pub fn shr_u16<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_srli_epi16::<BITS>(self.inner()) }, self.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let shifted = vshrq_n_u16::<BITS>(vreinterpretq_u16_u8(self.inner()));
                    vreinterpretq_u8_u16(shifted)
                };
                Self::new(inner, self.cpu())
            }
            _ => unused!(self)
        })
    }
}

/// Arithmetic ops.
impl<C: CpuSimd> Vec128<C> {
    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_add_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vaddq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Add either another [`Vec128`] or a `u8` to every byte of this [`Vec128`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_adds_epu8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vqaddq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_sub_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vsubq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Subtract either another [`Vec128`] or a `u8` from every byte of this [`Vec128`],
    /// stopping at 0.
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_subs_epu8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vqsubq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Add either another [`Vec128`] or a `u32` to every `u32` of this [`Vec128`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn add_u32(self, other: impl Operand128<u32, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_add_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let sum = vaddq_u32(
                        vreinterpretq_u32_u8(self.inner()),
                        vreinterpretq_u32_u8(other.inner()),
                    );
                    vreinterpretq_u8_u32(sum)
                };
                Self::new(inner, self.cpu())
            }
            _ => unused!(self, other)
        })
    }
}

/// Comparison.
impl<C: CpuSimd> Vec128<C> {
    /// Compare every byte of this [`Vec128`] to either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand128<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_cmpeq_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vceqq_u8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Check if every byte of this [`Vec128`] equals either another [`Vec128`] or a `u8`.
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand128<u8, C>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand128<i8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_cmpgt_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let result = unsafe {
                    vcgtq_s8(vreinterpretq_s8_u8(self.inner()), vreinterpretq_s8_u8(other.inner()))
                };
                Self::new(result, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Compare every byte of this [`Vec128`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec128`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand128<i8, C>) -> Self {
        other.into_vec128(self.cpu()).gt_i8(self)
    }
}

/// Bytewise ops.
impl<C: CpuSimd> Vec128<C> {
    /// Shift the whole of this [`Vec128`] right by `BYTES` bytes, filling with zeros.
    ///
    /// Byte `N` moves to byte 0.
    #[inline(always)]
    pub fn shr_bytes<const BYTES: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_srli_si128::<BYTES>(self.inner()) }, self.cpu())
            }
            Neon => {
                let zero = Self::zero(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vextq_u8::<BYTES>(self.inner(), zero.inner()) }, self.cpu())
            }
            _ => unused!(self)
        })
    }

    /// Rearrange the bytes of this [`Vec128`].
    ///
    /// For each byte `i`:
    ///
    /// - If `control[i]` is less than 16, `result[i]` is `self[control[i]]`.
    /// - If the top bit of `control[i]` is set, `result[i]` is 0.
    /// - Otherwise, `result[i]` depends on the CPU.
    ///   It's `self[control[i] & 0x0F]` on AVX2, and 0 on Neon.
    ///
    /// Often used as a 16-entry lookup table, with `self` as the table.
    #[inline(always)]
    pub fn shuffle_u8s(self, control: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSSE3
                Self::new(unsafe { _mm_shuffle_epi8(self.inner(), control.inner()) }, self.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(unsafe { vqtbl1q_u8(self.inner(), control.inner()) }, self.cpu())
            }
            _ => unused!(self, control)
        })
    }

    /// Combine the bytes of this [`Vec128`] and either another [`Vec128`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs:
    ///
    /// - AVX2: Each byte comes from `other` if the top bit of the same byte of `mask` is set,
    ///   otherwise from `self`. Other bits of `mask` are ignored.
    /// - Neon: Each bit comes from `other` if the same bit of `mask` is set, otherwise from `self`.
    #[inline(always)]
    pub fn blend(self, other: impl Operand128<u8, C>, mask: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE4.1
                Self::new(
                    unsafe { _mm_blendv_epi8(self.inner(), other.inner(), mask.inner()) },
                    self.cpu(),
                )
            }
            Neon => {
                let other = other.into_vec128(self.cpu());
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                Self::new(
                    unsafe { vbslq_u8(mask.inner(), other.inner(), self.inner()) },
                    self.cpu(),
                )
            }
            _ => unused!(self, other, mask)
        })
    }
}

/// `u32`-wise ops.
impl<C: CpuSimd> Vec128<C> {
    /// Interleave the low 2 `u32`s of this [`Vec128`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_lo_u32(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_unpacklo_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let zipped = vzip1q_u32(
                        vreinterpretq_u32_u8(self.inner()),
                        vreinterpretq_u32_u8(other.inner()),
                    );
                    vreinterpretq_u8_u32(zipped)
                };
                Self::new(inner, self.cpu())
            }
            _ => unused!(self, other)
        })
    }

    /// Interleave the high 2 `u32`s of this [`Vec128`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_hi_u32(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE2
                Self::new(unsafe { _mm_unpackhi_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let zipped = vzip2q_u32(
                        vreinterpretq_u32_u8(self.inner()),
                        vreinterpretq_u32_u8(other.inner()),
                    );
                    vreinterpretq_u8_u32(zipped)
                };
                Self::new(inner, self.cpu())
            }
            _ => unused!(self, other)
        })
    }
}

/// `u64`-wise ops.
impl<C: CpuSimd> Vec128<C> {
    /// Get one of the 2 `u64`s of this [`Vec128`].
    ///
    /// `INDEX` is 0 for the low 8 bytes, or 1 for the high 8 bytes.
    #[inline(always)]
    pub fn get_u64<const INDEX: i32>(self) -> u64 {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec128<Avx2>`, so CPU supports AVX2,
                // which includes SSE4.1
                unsafe { _mm_extract_epi64::<INDEX>(self.inner()).cast_unsigned() }
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `self` is a `Vec128<Neon>`, so CPU supports Neon
                unsafe { vgetq_lane_u64::<INDEX>(vreinterpretq_u64_u8(self.inner())) }
            }
            _ => unused!(self)
        })
    }
}

// Operators `!`, `&`, `|`, `^`, `&=`, `|=` and `^=`, which call `not`, `and`, `or` and `xor`.
// As with those methods, the right-hand side can be a vector or a `u8`, so `v & 0x0f` works.

impl<C: CpuSimd> Not for Vec128<C> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec128::not(self)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitAnd<T> for Vec128<C> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitAndAssign<T> for Vec128<C> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitOr<T> for Vec128<C> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitOrAssign<T> for Vec128<C> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitXor<T> for Vec128<C> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<C: CpuSimd, T: Operand128<u8, C>> BitXorAssign<T> for Vec128<C> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}

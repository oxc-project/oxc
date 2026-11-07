//! [`Vec256`], a 256-bit SIMD vector which is generic over the CPU type.
//!
//! The methods defined here are the ones which every CPU type has.
//! Each method chooses its implementation depending on the CPU type, using [`cpu_select!`].
//!
//! [`avx2`] and [`neon`] modules add further methods which are exclusive to specific
//! CPU feature sets, e.g. [`movemask`] on x86_64.
//!
//! Neon registers are 128 bits, so on aarch64 a [`Vec256`] is a pair of registers,
//! and most of its methods work on each half separately.
//!
//! The `!`, `&`, `|` and `^` operators for [`Vec256`] are implemented here too.
//!
//! [`avx2`]: super::avx2
//! [`neon`]: super::neon
//! [`movemask`]: Vec256::movemask

// These methods must be inlined to get the `target_feature`s of the functions calling them,
// otherwise the intrinsics also can't be inlined into them - 2 function calls for each method
#![expect(clippy::inline_always)]

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::{
    CpuSimd, Vec128, VecData256,
    arrays::IntArray256,
    macros::{cpu_select, unused},
    operand::Operand256,
};

/// 256-bit SIMD vector.
///
/// Holds a `C: CpuSimd`, so a vector can only exist if the CPU supports that feature set.
/// `C` is zero-sized, so [`Vec256`] is the same size as the type it wraps.
///
/// See comment on [`Vec256Inner`] explaining the `Vec256Storage` type.
#[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
#[derive(Clone, Copy)]
pub struct Vec256<C: CpuSimd> {
    storage: C::Vec256Storage,
    cpu: C,
}

/// The platform's SIMD type, which [`Vec256`] methods work with.
///
/// [`__m256i`] on x86_64, and [`uint8x16x2_t`] on aarch64.
///
/// It doesn't depend on the CPU type, because only one CPU type's vectors can exist
/// on each platform. See [`CpuSimd`].
///
/// On other platforms, a [`Vec256`] can never be created, and this type is a [`VecData256`].
///
/// A [`Vec256`] doesn't store this type directly. It stores its CPU type's [`Vec256Storage`].
/// On the CPU type's own platform, that's this type, and the [`vec256_storage`] and
/// [`vec256_inner`] conversion methods are no-ops. [`Vec256::inner`] converts the storage type
/// to this inner type.
///
/// On platforms other than the CPU type's own, the storage type is a stand-in with the same
/// size and alignment as the CPU type's SIMD type, so the layout of a [`Vec256<Avx2>`]
/// or [`Vec256<Neon>`] is always the same on every platform. A [`Vec256`] can only be created on
/// a system which supports the CPU feature set, so the dummy conversion methods are unreachable.
///
/// [`__m256i`]: https://doc.rust-lang.org/std/arch/x86_64/struct.__m256i.html
/// [`uint8x16x2_t`]: https://doc.rust-lang.org/std/arch/aarch64/struct.uint8x16x2_t.html
/// [`Vec256Storage`]: super::CpuSimdPrivate::Vec256Storage
/// [`vec256_storage`]: super::CpuSimdPrivate::vec256_storage
/// [`vec256_inner`]: super::CpuSimdPrivate::vec256_inner
pub(super) type Vec256Inner = cfg_select! {
    target_arch = "x86_64" => __m256i,
    target_arch = "aarch64" => uint8x16x2_t,
    _ => VecData256,
};

// Methods which every CPU type has.
// A method which needs different code for different CPU types chooses it with `cpu_select!`.

/// Internal methods.
impl<C: CpuSimd> Vec256<C> {
    /// Create a [`Vec256`] wrapping `inner`.
    ///
    /// Internal method only.
    /// `pub(super)` so that the `avx2` and `neon` modules can use it.
    #[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub(super) fn new(inner: Vec256Inner, cpu: C) -> Self {
        Self { storage: C::vec256_storage(inner), cpu }
    }

    /// Get the platform's SIMD type which this [`Vec256`] wraps.
    ///
    /// Internal method only.
    /// `pub(super)` so that the `avx2` and `neon` modules can use it.
    #[cfg_attr(not(any(target_arch = "x86_64", target_arch = "aarch64")), expect(dead_code))]
    #[inline(always)]
    pub(super) fn inner(self) -> Vec256Inner {
        C::vec256_inner(self.storage)
    }
}

/// Accessors.
impl<C: CpuSimd> Vec256<C> {
    /// Get the [`CpuSimd`] token from this [`Vec256`].
    #[inline(always)]
    pub fn cpu(self) -> C {
        self.cpu
    }
}

/// Creation methods.
impl<C: CpuSimd> Vec256<C> {
    /// Create a [`Vec256`] with all bits zero.
    #[inline(always)]
    pub fn zero(cpu: C) -> Self {
        Self::splat(0, cpu)
    }

    /// Create a [`Vec256`] by repeating `n` 32 times.
    #[inline(always)]
    pub fn splat(n: u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2
                Self::new(unsafe { _mm256_set1_epi8(n.cast_signed()) }, cpu)
            }
            _ => {
                let half = Vec128::splat(n, cpu);
                Self::from_halves(half, half)
            }
        })
    }

    /// Create a [`Vec256`] by repeating `n` 8 times.
    #[inline(always)]
    pub fn splat_u32(n: u32, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2
                Self::new(unsafe { _mm256_set1_epi32(n.cast_signed()) }, cpu)
            }
            _ => {
                let half = Vec128::splat_u32(n, cpu);
                Self::from_halves(half, half)
            }
        })
    }

    /// Create a [`Vec256`] by repeating `v` twice.
    #[inline(always)]
    pub fn splat_v128(v: Vec128<C>) -> Self {
        cpu_select!(v.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `v` is a `Vec128<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_broadcastsi128_si256(v.inner()) }, v.cpu())
            }
            _ => {
                Self::from_halves(v, v)
            }
        })
    }

    /// Create a [`Vec256`] by zero-extending the low 8 bytes of `v` to 8 `u32`s.
    #[inline(always)]
    pub fn widen_u8_to_u32(v: Vec128<C>) -> Self {
        cpu_select!(v.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `v` is a `Vec128<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_cvtepu8_epi32(v.inner()) }, v.cpu())
            }
            Neon => {
                // SAFETY: `Neon` arm only runs if `v` is a `Vec128<Neon>`, so CPU supports Neon
                let inner = unsafe {
                    let u16s = vmovl_u8(vget_low_u8(v.inner()));
                    let low = vmovl_u16(vget_low_u16(u16s));
                    let high = vmovl_u16(vget_high_u16(u16s));
                    uint8x16x2_t(vreinterpretq_u8_u32(low), vreinterpretq_u8_u32(high))
                };
                Self::new(inner, v.cpu())
            }
            _ => unused!(v)
        })
    }
}

/// Loads and stores.
impl<C: CpuSimd> Vec256<C> {
    /// Read a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must be valid for reading 32 bytes.
    /// - `ptr` must be aligned on 32.
    #[inline(always)]
    pub unsafe fn load(ptr: *const u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
                let ptr = ptr.cast::<__m256i>();

                // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes, and aligned.
                // `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2.
                Self::new(unsafe { _mm256_load_si256(ptr) }, cpu)
            }
            _ => {
                // Neon has no separate aligned load instruction,
                // so this is the same as `load_unaligned`.
                // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
                unsafe { Self::load_unaligned(ptr, cpu) }
            }
        })
    }

    /// Read a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for reading 32 bytes.
    #[inline(always)]
    pub unsafe fn load_unaligned(ptr: *const u8, cpu: C) -> Self {
        cpu_select!(cpu, {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m256i>();

                // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
                // `Avx2` arm only runs if `cpu` is an `Avx2`, so CPU supports AVX2.
                Self::new(unsafe { _mm256_loadu_si256(ptr) }, cpu)
            }
            Neon => {
                // This is all `vld1q_u8_x2` does. Calling it would give the same instructions,
                // but it's a function which the compiler has to instantiate and then inline.
                // SAFETY: Caller guarantees `ptr` is valid for reading 32 bytes.
                // `Neon` arm only runs if `cpu` is a `Neon`, so CPU supports Neon.
                Self::new(unsafe { ptr.cast::<uint8x16x2_t>().read_unaligned() }, cpu)
            }
            _ => unused!(ptr, cpu)
        })
    }

    /// Read a [`Vec256`] from a [`VecData256`], which is aligned on 32.
    #[inline(always)]
    pub fn read(data: &VecData256, cpu: C) -> Self {
        // SAFETY: `VecData256` guarantees `data.as_ptr()`
        // is valid for aligned reads of 32 bytes
        unsafe { Self::load(data.as_ptr(), cpu) }
    }

    /// Read a [`Vec256`] from a slice of integers, with no alignment requirement.
    ///
    /// `slice` can be any slice of integers which is 32 bytes in total
    /// (e.g. `&[u8; 32]`, `&[i64; 4]`).
    #[inline(always)]
    pub fn read_unaligned(slice: &impl IntArray256, cpu: C) -> Self {
        // SAFETY: `IntArray256` guarantees `slice.as_ptr()`
        // is valid for unaligned reads of 32 bytes
        unsafe { Self::load_unaligned(slice.as_ptr(), cpu) }
    }

    /// Store a [`Vec256`] at pointer `ptr`, which must be aligned.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must be valid for writing 32 bytes.
    /// - `ptr` must be aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub unsafe fn store(self, ptr: *mut u8) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "caller guarantees `ptr` is aligned")]
                let ptr = ptr.cast::<__m256i>();

                // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes, and aligned.
                // `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2.
                unsafe { _mm256_store_si256(ptr, self.inner()) }
            }
            _ => {
                // Neon has no separate aligned store instruction,
                // so this is the same as `store_unaligned`.
                // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
                unsafe { self.store_unaligned(ptr) }
            }
        });
    }

    /// Store a [`Vec256`] at pointer `ptr`, with no alignment requirement.
    ///
    /// # SAFETY
    ///
    /// `ptr` must be valid for writing 32 bytes.
    #[inline(always)]
    pub unsafe fn store_unaligned(self, ptr: *mut u8) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                #[expect(clippy::cast_ptr_alignment, reason = "intrinsic doesn't need alignment")]
                let ptr = ptr.cast::<__m256i>();

                // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
                // `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2.
                unsafe { _mm256_storeu_si256(ptr, self.inner()) }
            }
            Neon => {
                // This is all `vst1q_u8_x2` does. Calling it would give the same instructions,
                // but it's a function which the compiler has to instantiate and then inline.
                // SAFETY: Caller guarantees `ptr` is valid for writing 32 bytes.
                unsafe { ptr.cast::<uint8x16x2_t>().write_unaligned(self.inner()) }
            }
            _ => unused!(self, ptr)
        });
    }

    /// Write a [`Vec256`] into a [`VecData256`], which is aligned on 32.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn write(self, dst: &mut VecData256) {
        // SAFETY: `VecData256` guarantees `dst.as_mut_ptr()`
        // is valid for aligned writes of 32 bytes
        unsafe { self.store(dst.as_mut_ptr()) };
    }
}

/// Bitwise ops.
impl<C: CpuSimd> Vec256<C> {
    /// Check if all the bits of this [`Vec256`] are zero.
    #[inline(always)]
    pub fn is_zero(self) -> bool {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                unsafe { _mm256_testz_si256(self.inner(), self.inner()) != 0 }
            }
            _ => {
                let (low, high) = self.into_halves();
                (low | high).is_zero()
            }
        })
    }

    /// Invert all the bits of this [`Vec256`].
    #[inline(always)]
    pub fn not(self) -> Self {
        self.xor(Self::splat(0xFF, self.cpu()))
    }

    /// AND this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn and(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_and_si256(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low & other_low, high & other_high)
            }
        })
    }

    /// OR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn or(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_or_si256(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low | other_low, high | other_high)
            }
        })
    }

    /// XOR this [`Vec256`] with either another [`Vec256`] or a `u8`.
    #[inline(always)]
    pub fn xor(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_xor_si256(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low ^ other_low, high ^ other_high)
            }
        })
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] left by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    pub fn shl_u16<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_slli_epi16::<BITS>(self.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                Self::from_halves(low.shl_u16::<BITS>(), high.shl_u16::<BITS>())
            }
        })
    }

    /// Shift each of the 16 `u16`s of this [`Vec256`] right by `BITS` bits.
    ///
    /// `BITS` must be between 1 and 15 (inclusive).
    #[inline(always)]
    pub fn shr_u16<const BITS: i32>(self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_srli_epi16::<BITS>(self.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                Self::from_halves(low.shr_u16::<BITS>(), high.shr_u16::<BITS>())
            }
        })
    }
}

/// Arithmetic ops.
impl<C: CpuSimd> Vec256<C> {
    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn add_u8(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_add_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low.add_u8(other_low), high.add_u8(other_high))
            }
        })
    }

    /// Add either another [`Vec256`] or a `u8` to every byte of this [`Vec256`],
    /// stopping at 255.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_add_u8(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_adds_epu8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(
                    low.saturating_add_u8(other_low),
                    high.saturating_add_u8(other_high),
                )
            }
        })
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// wrapping on overflow.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn sub_u8(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_sub_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low.sub_u8(other_low), high.sub_u8(other_high))
            }
        })
    }

    /// Subtract either another [`Vec256`] or a `u8` from every byte of this [`Vec256`],
    /// stopping at 0.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn saturating_sub_u8(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_subs_epu8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(
                    low.saturating_sub_u8(other_low),
                    high.saturating_sub_u8(other_high),
                )
            }
        })
    }

    /// Add either another [`Vec256`] or a `u32` to every `u32` of this [`Vec256`],
    /// wrapping on overflow.
    #[inline(always)]
    pub fn add_u32(self, other: impl Operand256<u32, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_add_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low.add_u32(other_low), high.add_u32(other_high))
            }
        })
    }
}

/// Comparison.
impl<C: CpuSimd> Vec256<C> {
    /// Compare every byte of this [`Vec256`] to either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result is `0xFF` if the bytes are equal, otherwise `0x00`.
    #[inline(always)]
    pub fn eq_u8(self, other: impl Operand256<u8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_cmpeq_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low.eq_u8(other_low), high.eq_u8(other_high))
            }
        })
    }

    /// Check if every byte of this [`Vec256`] equals either another [`Vec256`] or a `u8`.
    #[cfg_attr(not(test), expect(dead_code))]
    #[inline(always)]
    pub fn all_eq_u8(self, other: impl Operand256<u8, C>) -> bool {
        self.xor(other).is_zero()
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is greater, otherwise `0x00`.
    #[inline(always)]
    pub fn gt_i8(self, other: impl Operand256<i8, C>) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_cmpgt_epi8(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                Self::from_halves(low.gt_i8(other_low), high.gt_i8(other_high))
            }
        })
    }

    /// Compare every byte of this [`Vec256`] to `other`, as signed integers.
    /// `other` is either an `i8`, or another [`Vec256`] to compare byte by byte.
    ///
    /// Each byte of the result is `0xFF` if the byte in `self` is less, otherwise `0x00`.
    #[inline(always)]
    pub fn lt_i8(self, other: impl Operand256<i8, C>) -> Self {
        other.into_vec256(self.cpu()).gt_i8(self)
    }
}

/// Bytewise ops.
impl<C: CpuSimd> Vec256<C> {
    /// Combine the bytes of this [`Vec256`] and either another [`Vec256`] or a `u8`.
    ///
    /// Each byte of the result comes from `other` if the same byte of `mask` is `0xFF`,
    /// or from `self` if it's `0x00`.
    /// Other values in `mask` give different results on different CPUs:
    ///
    /// - AVX2: Each byte comes from `other` if the top bit of the same byte of `mask` is set,
    ///   otherwise from `self`. Other bits of `mask` are ignored.
    /// - Neon: Each bit comes from `other` if the same bit of `mask` is set, otherwise from `self`.
    #[inline(always)]
    pub fn blend(self, other: impl Operand256<u8, C>, mask: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                let other = other.into_vec256(self.cpu());
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(
                    unsafe { _mm256_blendv_epi8(self.inner(), other.inner(), mask.inner()) },
                    self.cpu(),
                )
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_vec256(self.cpu()).into_halves();
                let (mask_low, mask_high) = mask.into_halves();
                Self::from_halves(low.blend(other_low, mask_low), high.blend(other_high, mask_high))
            }
        })
    }
}

/// `u32`-wise ops.
impl<C: CpuSimd> Vec256<C> {
    /// Interleave the low 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s0, o0, s1, o1, s4, o4, s5, o5]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_lo_u32(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_unpacklo_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_halves();
                Self::from_halves(low.unpack_lo_u32(other_low), high.unpack_lo_u32(other_high))
            }
        })
    }

    /// Interleave the high 2 `u32`s of each 128-bit half of this [`Vec256`] and `other`.
    ///
    /// The result is `[s2, o2, s3, o3, s6, o6, s7, o7]`,
    /// where `sN` and `oN` are the `u32`s at index `N` in `self` and `other`.
    #[inline(always)]
    pub fn unpack_hi_u32(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_unpackhi_epi32(self.inner(), other.inner()) }, self.cpu())
            }
            _ => {
                let (low, high) = self.into_halves();
                let (other_low, other_high) = other.into_halves();
                Self::from_halves(low.unpack_hi_u32(other_low), high.unpack_hi_u32(other_high))
            }
        })
    }
}

/// Operations on 128-bit halves.
impl<C: CpuSimd> Vec256<C> {
    /// Create a [`Vec256`] from its low and high 128 bits.
    #[inline(always)]
    pub fn from_halves(low: Vec128<C>, high: Vec128<C>) -> Self {
        cpu_select!(low.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `low` is a `Vec128<Avx2>`, so CPU supports AVX2
                Self::new(unsafe { _mm256_set_m128i(high.inner(), low.inner()) }, low.cpu())
            }
            Neon => {
                Self::new(uint8x16x2_t(low.inner(), high.inner()), low.cpu())
            }
            _ => unused!(low, high)
        })
    }

    /// Get the low and high 128 bits of this [`Vec256`].
    #[inline(always)]
    pub fn into_halves(self) -> (Vec128<C>, Vec128<C>) {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                let (low, high) = unsafe {
                    (
                        _mm256_castsi256_si128(self.inner()),
                        _mm256_extracti128_si256::<1>(self.inner()),
                    )
                };
                (Vec128::new(low, self.cpu()), Vec128::new(high, self.cpu()))
            }
            Neon => {
                (Vec128::new(self.inner().0, self.cpu()), Vec128::new(self.inner().1, self.cpu()))
            }
            _ => unused!(self)
        })
    }

    /// Get a [`Vec256`] made of the low 128 bits of this [`Vec256`],
    /// followed by the low 128 bits of `other`.
    #[inline(always)]
    pub fn join_lo_halves(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(
                    unsafe { _mm256_permute2x128_si256::<0x20>(self.inner(), other.inner()) },
                    self.cpu(),
                )
            }
            _ => {
                Self::from_halves(self.into_halves().0, other.into_halves().0)
            }
        })
    }

    /// Get a [`Vec256`] made of the high 128 bits of this [`Vec256`],
    /// followed by the high 128 bits of `other`.
    #[inline(always)]
    pub fn join_hi_halves(self, other: Self) -> Self {
        cpu_select!(self.cpu(), {
            Avx2 => {
                // SAFETY: `Avx2` arm only runs if `self` is a `Vec256<Avx2>`, so CPU supports AVX2
                Self::new(
                    unsafe { _mm256_permute2x128_si256::<0x31>(self.inner(), other.inner()) },
                    self.cpu(),
                )
            }
            _ => {
                Self::from_halves(self.into_halves().1, other.into_halves().1)
            }
        })
    }
}

// Operators `!`, `&`, `|`, `^`, `&=`, `|=` and `^=`, which call `not`, `and`, `or` and `xor`.
// As with those methods, the right-hand side can be a vector or a `u8`, so `v & 0x0f` works.

impl<C: CpuSimd> Not for Vec256<C> {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Vec256::not(self)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitAnd<T> for Vec256<C> {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self {
        self.and(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitAndAssign<T> for Vec256<C> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        *self = self.and(rhs);
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitOr<T> for Vec256<C> {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self {
        self.or(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitOrAssign<T> for Vec256<C> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        *self = self.or(rhs);
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitXor<T> for Vec256<C> {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self {
        self.xor(rhs)
    }
}

impl<C: CpuSimd, T: Operand256<u8, C>> BitXorAssign<T> for Vec256<C> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        *self = self.xor(rhs);
    }
}

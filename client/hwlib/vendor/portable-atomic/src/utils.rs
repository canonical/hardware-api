// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg_attr(not(all(test, feature = "float")), allow(dead_code, unused_macros))]

#[macro_use]
#[path = "gen/utils.rs"]
mod gen;

use core::sync::atomic::Ordering;

macro_rules! static_assert {
    ($cond:expr $(,)?) => {{
        let [] = [(); true as usize - $crate::utils::_assert_is_bool($cond) as usize];
    }};
}
pub(crate) const fn _assert_is_bool(v: bool) -> bool {
    v
}

macro_rules! static_assert_layout {
    ($atomic_type:ty, $value_type:ty) => {
        static_assert!(
            core::mem::align_of::<$atomic_type>() == core::mem::size_of::<$atomic_type>()
        );
        static_assert!(core::mem::size_of::<$atomic_type>() == core::mem::size_of::<$value_type>());
    };
}

// #[doc = concat!(...)] requires Rust 1.54
macro_rules! doc_comment {
    ($doc:expr, $($tt:tt)*) => {
        #[doc = $doc]
        $($tt)*
    };
}

// Adapted from https://github.com/BurntSushi/memchr/blob/2.4.1/src/memchr/x86/mod.rs#L9-L71.
/// # Safety
///
/// - the caller must uphold the safety contract for the function returned by $detect_body.
/// - the memory pointed by the function pointer returned by $detect_body must be visible from any threads.
///
/// The second requirement is always met if the function pointer is to the function definition.
/// (Currently, all uses of this macro in our code are in this case.)
#[allow(unused_macros)]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "powerpc64",
    all(target_arch = "x86_64", not(any(target_env = "sgx", miri))),
))]
macro_rules! ifunc {
    (unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)? { $($detect_body:tt)* }) => {{
        type FnTy = unsafe fn($($arg_ty),*) $(-> $ret_ty)?;
        static FUNC: core::sync::atomic::AtomicPtr<()>
            = core::sync::atomic::AtomicPtr::new(detect as *mut ());
        #[cold]
        unsafe fn detect($($arg_pat: $arg_ty),*) $(-> $ret_ty)? {
            let func: FnTy = { $($detect_body)* };
            FUNC.store(func as *mut (), core::sync::atomic::Ordering::Relaxed);
            // SAFETY: the caller must uphold the safety contract for the function returned by $detect_body.
            unsafe { func($($arg_pat),*) }
        }
        // SAFETY: `FnTy` is a function pointer, which is always safe to transmute with a `*mut ()`.
        // (To force the caller to use unsafe block for this macro, do not use
        // unsafe block here.)
        let func = {
            core::mem::transmute::<*mut (), FnTy>(FUNC.load(core::sync::atomic::Ordering::Relaxed))
        };
        // SAFETY: the caller must uphold the safety contract for the function returned by $detect_body.
        // (To force the caller to use unsafe block for this macro, do not use
        // unsafe block here.)
        func($($arg_pat),*)
    }};
}

#[allow(unused_macros)]
#[cfg(not(portable_atomic_no_outline_atomics))]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "powerpc64",
    all(target_arch = "x86_64", not(any(target_env = "sgx", miri))),
))]
macro_rules! fn_alias {
    (
        $(#[$($fn_attr:tt)*])*
        $vis:vis unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)?;
        $(#[$($alias_attr:tt)*])*
        $new:ident = $from:ident($($last_args:tt)*);
        $($rest:tt)*
    ) => {
        $(#[$($fn_attr)*])*
        $(#[$($alias_attr)*])*
        $vis unsafe fn $new($($arg_pat: $arg_ty),*) $(-> $ret_ty)? {
            // SAFETY: the caller must uphold the safety contract.
            unsafe { $from($($arg_pat,)* $($last_args)*) }
        }
        fn_alias! {
            $(#[$($fn_attr)*])*
            $vis unsafe fn($($arg_pat: $arg_ty),*) $(-> $ret_ty)?;
            $($rest)*
        }
    };
    (
        $(#[$($attr:tt)*])*
        $vis:vis unsafe fn($($arg_pat:ident: $arg_ty:ty),*) $(-> $ret_ty:ty)?;
    ) => {}
}

/// Make the given function const if the given condition is true.
macro_rules! const_fn {
    (
        const_if: #[cfg($($cfg:tt)+)];
        $(#[$($attr:tt)*])*
        $vis:vis const fn $($rest:tt)*
    ) => {
        #[cfg($($cfg)+)]
        $(#[$($attr)*])*
        $vis const fn $($rest)*
        #[cfg(not($($cfg)+))]
        $(#[$($attr)*])*
        $vis fn $($rest)*
    };
}

/// Implements `core::fmt::Debug` and `serde::{Serialize, Deserialize}` (when serde
/// feature is enabled) for atomic bool, integer, or float.
macro_rules! impl_debug_and_serde {
    ($atomic_type:ident) => {
        impl fmt::Debug for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // fmt is not hot path
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // std atomic types use Relaxed in Debug::fmt: https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L2024
                fmt::Debug::fmt(&self.load(Ordering::Relaxed), f)
            }
        }
        #[cfg(feature = "serde")]
        #[cfg_attr(portable_atomic_doc_cfg, doc(cfg(feature = "serde")))]
        impl serde::ser::Serialize for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // serde doesn't use inline on std atomic's Serialize/Deserialize impl
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                // https://github.com/serde-rs/serde/blob/v1.0.152/serde/src/ser/impls.rs#L958-L959
                self.load(Ordering::Relaxed).serialize(serializer)
            }
        }
        #[cfg(feature = "serde")]
        #[cfg_attr(portable_atomic_doc_cfg, doc(cfg(feature = "serde")))]
        impl<'de> serde::de::Deserialize<'de> for $atomic_type {
            #[allow(clippy::missing_inline_in_public_items)] // serde doesn't use inline on std atomic's Serialize/Deserialize impl
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                serde::de::Deserialize::deserialize(deserializer).map(Self::new)
            }
        }
    };
}

// We do not provide `nand` because it cannot be optimized on neither x86 nor MSP430.
// https://godbolt.org/z/7TzjKqYvE
macro_rules! impl_default_no_fetch_ops {
    ($atomic_type:ident, bool) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn and(&self, val: bool, order: Ordering) {
                self.fetch_and(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn or(&self, val: bool, order: Ordering) {
                self.fetch_or(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn xor(&self, val: bool, order: Ordering) {
                self.fetch_xor(val, order);
            }
        }
    };
    ($atomic_type:ident, $int_type:ident) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn add(&self, val: $int_type, order: Ordering) {
                self.fetch_add(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn sub(&self, val: $int_type, order: Ordering) {
                self.fetch_sub(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn and(&self, val: $int_type, order: Ordering) {
                self.fetch_and(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn or(&self, val: $int_type, order: Ordering) {
                self.fetch_or(val, order);
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn xor(&self, val: $int_type, order: Ordering) {
                self.fetch_xor(val, order);
            }
        }
    };
}
macro_rules! impl_default_bit_opts {
    ($atomic_type:ident, $int_type:ident) => {
        impl $atomic_type {
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_set(&self, bit: u32, order: Ordering) -> bool {
                let mask = <$int_type>::wrapping_shl(1, bit);
                self.fetch_or(mask, order) & mask != 0
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_clear(&self, bit: u32, order: Ordering) -> bool {
                let mask = <$int_type>::wrapping_shl(1, bit);
                self.fetch_and(!mask, order) & mask != 0
            }
            #[inline]
            #[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
            pub(crate) fn bit_toggle(&self, bit: u32, order: Ordering) -> bool {
                let mask = <$int_type>::wrapping_shl(1, bit);
                self.fetch_xor(mask, order) & mask != 0
            }
        }
    };
}

// This just outputs the input as is, but can be used like an item-level block by using it with cfg.
macro_rules! items {
    ($($tt:tt)*) => {
        $($tt)*
    };
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3155
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_load_ordering(order: Ordering) {
    match order {
        Ordering::Acquire | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Release => panic!("there is no such thing as a release load"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release load"),
        _ => unreachable!("{:?}", order),
    }
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3140
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_store_ordering(order: Ordering) {
    match order {
        Ordering::Release | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Acquire => panic!("there is no such thing as an acquire store"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release store"),
        _ => unreachable!("{:?}", order),
    }
}

// https://github.com/rust-lang/rust/blob/1.70.0/library/core/src/sync/atomic.rs#L3221
#[inline]
#[cfg_attr(all(debug_assertions, not(portable_atomic_no_track_caller)), track_caller)]
pub(crate) fn assert_compare_exchange_ordering(success: Ordering, failure: Ordering) {
    match success {
        Ordering::AcqRel
        | Ordering::Acquire
        | Ordering::Relaxed
        | Ordering::Release
        | Ordering::SeqCst => {}
        _ => unreachable!("{:?}, {:?}", success, failure),
    }
    match failure {
        Ordering::Acquire | Ordering::Relaxed | Ordering::SeqCst => {}
        Ordering::Release => panic!("there is no such thing as a release failure ordering"),
        Ordering::AcqRel => panic!("there is no such thing as an acquire-release failure ordering"),
        _ => unreachable!("{:?}, {:?}", success, failure),
    }
}

// https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2016/p0418r2.html
// https://github.com/rust-lang/rust/pull/98383
#[allow(dead_code)]
#[inline]
pub(crate) fn upgrade_success_ordering(success: Ordering, failure: Ordering) -> Ordering {
    match (success, failure) {
        (Ordering::Relaxed, Ordering::Acquire) => Ordering::Acquire,
        (Ordering::Release, Ordering::Acquire) => Ordering::AcqRel,
        (_, Ordering::SeqCst) => Ordering::SeqCst,
        _ => success,
    }
}

/// Zero-extends the given 32-bit pointer to `MaybeUninit<u64>`.
/// This is used for 64-bit architecture's 32-bit ABI (e.g., AArch64 ILP32 ABI).
/// See ptr_reg! macro in src/gen/utils.rs for details.
#[cfg(not(portable_atomic_no_asm_maybe_uninit))]
#[cfg(target_pointer_width = "32")]
#[allow(dead_code)]
#[inline]
pub(crate) fn zero_extend64_ptr(v: *mut ()) -> core::mem::MaybeUninit<u64> {
    #[repr(C)]
    struct ZeroExtended {
        #[cfg(target_endian = "big")]
        pad: *mut (),
        v: *mut (),
        #[cfg(target_endian = "little")]
        pad: *mut (),
    }
    // SAFETY: we can safely transmute any 64-bit value to MaybeUninit<u64>.
    unsafe { core::mem::transmute(ZeroExtended { v, pad: core::ptr::null_mut() }) }
}

#[allow(dead_code)]
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "powerpc64",
    target_arch = "s390x",
    target_arch = "x86_64",
))]
/// A 128-bit value represented as a pair of 64-bit values.
///
/// This type is `#[repr(C)]`, both fields have the same in-memory representation
/// and are plain old data types, so access to the fields is always safe.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union U128 {
    pub(crate) whole: u128,
    pub(crate) pair: Pair<u64>,
}
#[allow(dead_code)]
#[cfg(target_arch = "arm")]
/// A 64-bit value represented as a pair of 32-bit values.
///
/// This type is `#[repr(C)]`, both fields have the same in-memory representation
/// and are plain old data types, so access to the fields is always safe.
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) union U64 {
    pub(crate) whole: u64,
    pub(crate) pair: Pair<u32>,
}
#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct Pair<T: Copy> {
    // little endian order
    #[cfg(any(target_endian = "little", target_arch = "aarch64", target_arch = "arm"))]
    pub(crate) lo: T,
    pub(crate) hi: T,
    // big endian order
    #[cfg(not(any(target_endian = "little", target_arch = "aarch64", target_arch = "arm")))]
    pub(crate) lo: T,
}

#[allow(dead_code)]
type MinWord = u32;
#[cfg(target_arch = "riscv32")]
type RegSize = u32;
#[cfg(target_arch = "riscv64")]
type RegSize = u64;
// Adapted from https://github.com/taiki-e/atomic-maybe-uninit/blob/v0.3.0/src/utils.rs#L210.
// Helper for implementing sub-word atomic operations using word-sized LL/SC loop or CAS loop.
//
// Refs: https://github.com/llvm/llvm-project/blob/llvmorg-17.0.0-rc2/llvm/lib/CodeGen/AtomicExpandPass.cpp#L699
// (aligned_ptr, shift, mask)
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[allow(dead_code)]
#[inline]
pub(crate) fn create_sub_word_mask_values<T>(ptr: *mut T) -> (*mut MinWord, RegSize, RegSize) {
    use core::mem;
    const SHIFT_MASK: bool = !cfg!(any(
        target_arch = "riscv32",
        target_arch = "riscv64",
        target_arch = "loongarch64",
        target_arch = "s390x",
    ));
    let ptr_mask = mem::size_of::<MinWord>() - 1;
    let aligned_ptr = strict::with_addr(ptr, ptr as usize & !ptr_mask) as *mut MinWord;
    let ptr_lsb = if SHIFT_MASK {
        ptr as usize & ptr_mask
    } else {
        // We use 32-bit wrapping shift instructions in asm on these platforms.
        ptr as usize
    };
    let shift = if cfg!(any(target_endian = "little", target_arch = "s390x")) {
        ptr_lsb.wrapping_mul(8)
    } else {
        (ptr_lsb ^ (mem::size_of::<MinWord>() - mem::size_of::<T>())).wrapping_mul(8)
    };
    let mut mask: RegSize = (1 << (mem::size_of::<T>() * 8)) - 1; // !(0 as T) as RegSize
    if SHIFT_MASK {
        mask <<= shift;
    }
    (aligned_ptr, shift as RegSize, mask)
}

/// Emulate strict provenance.
///
/// Once strict_provenance is stable, migrate to the standard library's APIs.
#[cfg(any(miri, target_arch = "riscv32", target_arch = "riscv64"))]
#[allow(dead_code)]
pub(crate) mod strict {
    /// Replace the address portion of this pointer with a new address.
    #[inline]
    #[must_use]
    pub(crate) fn with_addr<T>(ptr: *mut T, addr: usize) -> *mut T {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        //
        // In the mean-time, this operation is defined to be "as if" it was
        // a wrapping_add, so we can emulate it as such. This should properly
        // restore pointer provenance even under today's compiler.
        let offset = addr.wrapping_sub(ptr as usize);

        // This is the canonical desugaring of this operation.
        (ptr as *mut u8).wrapping_add(offset) as *mut T
    }

    /// Run an operation of some kind on a pointer.
    #[inline]
    #[must_use]
    pub(crate) fn map_addr<T>(ptr: *mut T, f: impl FnOnce(usize) -> usize) -> *mut T {
        with_addr(ptr, f(ptr as usize))
    }
}

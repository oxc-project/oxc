use super::CHUNK_ALIGN;

/// Configuration trait for [`Arena`]s.
///
/// # Example
///
/// ```
/// # use oxc_allocator::__arena::{Arena, ArenaConfig};
///
/// struct MyArenaConfig;
///
/// impl ArenaConfig for MyArenaConfig {
///     const MIN_ALIGN: usize = 16;
/// }
///
/// type MyArena = Arena<MyArenaConfig>;
///
/// let arena = MyArena::new();
/// let u8_ref = arena.alloc(123u8);
/// // `u8_ref` is aligned on 16, even though `u8` only requires alignment of 1
/// assert!(std::ptr::from_ref(u8_ref) as usize % 16 == 0);
/// ```
///
/// [`Arena`]: super::Arena
pub trait ArenaConfig {
    /// Minimum alignment of allocations in the arena.
    ///
    /// Types with lower alignment than `MIN_ALIGN` will be aligned on `MIN_ALIGN`.
    ///
    /// If you expect all/most values allocated in the arena to have same alignment,
    /// setting `MIN_ALIGN` to that alignment makes allocation cheaper.
    ///
    /// Restrictions:
    ///
    /// * Cannot be 0.
    /// * Must be a power of 2.
    /// * Cannot be greater than [`CHUNK_ALIGN`].
    ///
    /// Breaking any of those restrictions will produce a compile-time error.
    const MIN_ALIGN: usize;
}

/// Assertions for invariants of [`ArenaConfig`].
///
/// Blanket implemented for all types that implement [`ArenaConfig`].
///
/// [`ArenaConfigExt::ASSERTS`] must be referenced in all methods which create an [`Arena`],
/// to ensure it's impossible to create an `Arena` with an invalid config.
///
/// This trait is not exposed outside `arena` module. That's not for safety - blanket impl below means
/// it's impossible for a user to manually implement `ArenaConfigExt`, and override the assertions.
/// But just it doesn't need to be exposed, because it's not useful. So don't.
///
/// [`Arena`]: super::Arena
pub(super) trait ArenaConfigExt: ArenaConfig {
    const ASSERTS: () = {
        assert!(Self::MIN_ALIGN > 0, "`MIN_ALIGN` cannot be 0");
        assert!(Self::MIN_ALIGN.is_power_of_two(), "`MIN_ALIGN` must be a power of 2");
        assert!(Self::MIN_ALIGN <= CHUNK_ALIGN, "`MIN_ALIGN` cannot be greater than `CHUNK_ALIGN`");
    };
}

impl<C: ArenaConfig> ArenaConfigExt for C {}

/// Default [`ArenaConfig`], with `MIN_ALIGN = 1`.
///
/// i.e. all types will be stored with only their required alignment.
pub struct ArenaConfigDefault;

impl ArenaConfig for ArenaConfigDefault {
    const MIN_ALIGN: usize = 1;
}

/// Default [`ArenaConfig`], with `MIN_ALIGN = align_of::<usize>()`
///
/// i.e. all types will be stored with pointer alignment
/// (or greater if the type requires greater alignment).
pub struct ArenaConfigPointerAligned;

impl ArenaConfig for ArenaConfigPointerAligned {
    const MIN_ALIGN: usize = align_of::<usize>();
}

use super::{BASELINE, Cpu, DynCpu, apply, cpu_select, cpu_versions, to_dyn};

#[apply(cpu_versions)]
const fn add_one(x: u32, _cpu: impl Cpu) -> u32 {
    x + 1
}

#[apply(cpu_versions)]
const unsafe fn read_byte(ptr: *const u8, _cpu: impl Cpu) -> u8 {
    // SAFETY: Caller guarantees `ptr` is valid for reads
    unsafe { *ptr }
}

struct Counter(u32);

impl Counter {
    #[apply(cpu_versions)]
    const fn plus(&self, x: u32, _cpu: impl Cpu) -> u32 {
        self.0 + x
    }

    #[apply(cpu_versions)]
    const unsafe fn plus_byte(&self, ptr: *const u8, _cpu: impl Cpu) -> u32 {
        // SAFETY: Caller guarantees `ptr` is valid for reads
        self.0 + unsafe { *ptr } as u32
    }
}

/// [`BASELINE`] as a [`DynCpu`].
const BASELINE_DYN: DynCpu = to_dyn(BASELINE);

/// Calls to `const` functions made with `cpu_versions!`, evaluated at compile time.
const AT_COMPILE_TIME: [u32; 5] = {
    let bytes: [u8; 2] = [1, 2];
    [
        add_one(1, BASELINE),
        add_one(10, BASELINE_DYN),
        Counter(100).plus(1, BASELINE),
        // SAFETY: Pointer is valid for reads of 1 byte
        unsafe { read_byte(&raw const bytes[1], BASELINE) } as u32,
        // SAFETY: Pointer is valid for reads of 1 byte
        unsafe { Counter(100).plus_byte(&raw const bytes[1], BASELINE) },
    ]
};

#[test]
fn const_fn_at_compile_time() {
    assert_eq!(AT_COMPILE_TIME, [2, 11, 101, 2, 102]);
}

#[test]
fn const_fn_at_runtime() {
    let cpu = DynCpu::detect();
    let bytes: [u8; 2] = [1, 2];

    assert_eq!(add_one(1, cpu), 2);
    assert_eq!(add_one(1, BASELINE), 2);
    assert_eq!(Counter(100).plus(1, cpu), 101);
    // SAFETY: Pointer is valid for reads of 1 byte
    assert_eq!(unsafe { read_byte(&raw const bytes[1], cpu) }, 2);
    // SAFETY: Pointer is valid for reads of 1 byte
    assert_eq!(unsafe { Counter(100).plus_byte(&raw const bytes[1], cpu) }, 102);

    cpu_select!(cpu, {
        Avx2(cpu) => assert_eq!(add_one(1, cpu), 2),
        Neon(cpu) => assert_eq!(add_one(1, cpu), 2),
        Generic(cpu) => assert_eq!(add_one(1, cpu), 2),
    });
}

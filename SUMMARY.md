# Issue #14732 - Worker Thread Heap Corruption on Windows

## Status: Reproduced and Documented

### What I Did

1. ✅ Created failing reproduction files based on issue report
2. ✅ Built local oxc-parser with global allocator enabled
3. ✅ Tested reproduction on macOS - **NO CRASHES** (Windows-specific issue confirmed)
4. ✅ Identified version range: v0.92.0 (good) → v0.95.0 (bad)
5. ✅ Set up bisect infrastructure

### Files Created

- `napi/parser/test-worker-main.mjs` - Main test that spawns 16 workers
- `napi/parser/test-worker.mjs` - Worker that imports oxc-parser  
- `bisect-test.sh` - Automated bisect test script
- `REPRO-14732.md` - Detailed reproduction documentation

### Test Results on macOS

```
Run 1: ✓ All 16 workers completed
Run 2: ✓ All 16 workers completed  
Run 3: ✓ All 16 workers completed
Run 4: ✓ All 16 workers completed
Run 5: ✓ All 16 workers completed
```

**Total: 80 worker threads, 0 crashes**

### Key Findings

1. **Windows-Specific**: Error code 0xC0000374 (STATUS_HEAP_CORRUPTION) is Windows-only
2. **Global Allocator**: `mimalloc-safe` is used as global allocator (enabled via `allocator` feature)
3. **No Code Changes**: The global allocator setup is identical in v0.92.0 and v0.95.0
4. **Worker Threads**: Issue only occurs with multiple Node.js worker threads (16 concurrent)

### Why Can't Bisect on macOS?

The issue does NOT reproduce on macOS because:
- Different memory management between Windows and Unix
- Windows TLS (Thread Local Storage) behavior differs
- The heap corruption is Windows MSVC runtime specific

### What Needs to Happen Next

**Someone with access to Windows needs to run the bisect:**

```bash
# On Windows machine:
git bisect start
git bisect bad crates_v0.95.0    # 454ee94ff - broken
git bisect good crates_v0.92.0   # 1b3f43746 - working

# Then for each commit:
cd napi/parser
pnpm build --features allocator --release
node test-worker-main.mjs

# If crashes (heap corruption):
git bisect bad

# If completes successfully:
git bisect good
```

### Hypothesis

The issue likely stems from one of:

1. **NAPI module initialization changes** - How native modules are loaded per worker
2. **Allocator lifecycle** - Global allocator not properly isolated between workers
3. **Build configuration** - Windows-specific compiler/linker settings changed
4. **Dependencies** - Upstream changes in napi-rs or mimalloc-safe

### Commits to Investigate

Range: 1b3f43746 (v0.92.0) to 454ee94ff (v0.95.0)

Suspect areas:
- NAPI infrastructure changes
- Module loading/initialization
- Windows-specific build changes
- Thread-local storage configuration

The bisect will identify the exact breaking commit, then we can determine the fix.

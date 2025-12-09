// Test case for removing unused import equals chains
// See: https://github.com/oxc-project/oxc/issues/8278

// These should all be kept (used chain)
import a = foo.a
import b = a.b
import c = b.c

// These should all be removed (unused chain)
import x = foo.x
import y = x.y
import z = y.z

export let bar = c

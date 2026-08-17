import { afterEach, describe, expect, it } from "vitest";
import { buffers, forgetBuffer, occupiedBufferCount } from "../src-js/plugins/lint.ts";

import type { BufferWithArrays } from "../src-js/plugins/types.ts";

function fakeBuffer(): BufferWithArrays {
  return new Uint8Array(8) as BufferWithArrays;
}

describe("forgetBuffer", () => {
  afterEach(() => {
    buffers.length = 0;
  });

  it("nulls the cached buffer at the given id", () => {
    const first = fakeBuffer();
    const second = fakeBuffer();
    buffers[0] = first;
    buffers[1] = second;

    forgetBuffer(0);

    expect(buffers[0]).toBeNull();
    expect(buffers[1]).toBe(second);
  });

  it("does not grow the cache for an unknown id", () => {
    forgetBuffer(99);
    expect(buffers).toHaveLength(0);
  });

  it("is a no-op when the slot is already empty", () => {
    buffers.push(null);
    forgetBuffer(0);
    expect(buffers[0]).toBeNull();
    expect(buffers).toHaveLength(1);
  });
});

describe("occupiedBufferCount", () => {
  afterEach(() => {
    buffers.length = 0;
  });

  it("counts only non-null entries", () => {
    buffers.push(fakeBuffer(), null, fakeBuffer());
    expect(occupiedBufferCount()).toBe(2);
    forgetBuffer(0);
    expect(occupiedBufferCount()).toBe(1);
    forgetBuffer(2);
    expect(occupiedBufferCount()).toBe(0);
  });
});

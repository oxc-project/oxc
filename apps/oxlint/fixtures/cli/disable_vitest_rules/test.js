// File should have a total of 1 error from the vitest rule, and 1 warning about an unnecessary disable.
import { vi } from 'vitest';

vi.useFakeTimers();

it('calls the callback after 1 second via advanceTimersByTime', () => {
  vi.advanceTimersByTime(1000);
})

test('plays video', () => {
  // oxlint-disable vitest/no-restricted-vi-methods
  vi.spyOn(audio, 'play'); // this is disabled by the block above, should be no error.
  // oxlint-enable vitest/no-restricted-vi-methods

  // oxlint-disable-next-line vitest/no-restricted-vi-methods
  const spy = vi.spyOn(video, 'play'); // this is disabled by the line above, should be no error.

  // This one should trigger a warning about an unnecessary disable:
  // oxlint-disable-next-line vitest/no-restricted-vi-methods
  video.play();

  // Next line should not have an error, as we disable the rule.
  vi.spyOn(audio, 'play'); // oxlint-disable-line vitest/no-restricted-vi-methods
})

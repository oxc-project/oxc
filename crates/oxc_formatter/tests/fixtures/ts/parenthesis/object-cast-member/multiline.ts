const where = (status: string): [string, string] =>
  (
    ({
      draft: ['a', 'b'],
      pending: ['c', 'd'],
      rejected: ['e', 'f'],
    }) as Record<string, [string, string]>
  )[status] ?? ['', ''];

const satisfiesLookup = (status: string) =>
  (
    ({
      draft: ['a', 'b'],
      pending: ['c', 'd'],
    }) satisfies Record<string, [string, string]>
  )[status];

const staticLookup = () =>
  (
    ({
      draft: ['a', 'b'],
    }) as Record<string, [string, string]>
  ).draft;

const optionalLookup = (status: string) =>
  (
    ({
      draft: ['a', 'b'],
    }) satisfies Record<string, [string, string]>
  )?.[status];

const commentedLookup = (status: string) =>
  (
    ({
      // Keep this comment with the property.
      draft: ['a', 'b'],
    }) as Record<string, [string, string]>
  )[status];

const directCast = () =>
  ({
    draft: ['a', 'b'],
  }) as Record<string, [string, string]>;

const directSatisfies = () =>
  ({
    draft: ['a', 'b'],
  }) satisfies Record<string, [string, string]>;

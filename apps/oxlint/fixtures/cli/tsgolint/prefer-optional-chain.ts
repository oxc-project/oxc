// prefer-optional-chain: The pattern `foo && foo.bar` should use optional chaining `foo?.bar`
declare const fooOptC: { bar: number } | null | undefined;
fooOptC && fooOptC.bar;

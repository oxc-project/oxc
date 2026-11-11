// A union broken out of a type alias's `=` expands to leading-`|` members right away.
// DIVERGES from Prettier (DIVERGENCES.md#union-annotation-flat-retry), which
// since 3.9 first retries the whole union FLAT on the indented next line;
// deliberately held at the 3.8 behavior (oxc-project/oxc#25841).
// `Narrow` diverges at printWidth 80 (flat next line fits), `Wide` at 100;
// at the respectively other width each is inline (80) or fully expanded (100) in both formatters.
type Narrow = "primary" | "danger" | "warning" | "muted" | "success" | "information";
type Wide = "primary" | "danger" | "warning" | "muted" | "success" | "information" | "highlighted-item";

// A member's comment stops the hug of `X | null`, so it expands the same way.
// Diverges at printWidth 80; inline at 100 in both formatters.
type MemberComment = /* c */ CrmOnlineRecordApiTypes.CustomerProjectDetailsResponse | null | void;

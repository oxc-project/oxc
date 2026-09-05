// A union broken out of its `:` annotation position expands to leading-`|`
// members right away.
// DIVERGES from Prettier (DIVERGENCES.md "union-annotation-flat-retry"), which
// since 3.9 first retries the whole union FLAT on the indented next line and
// only then expands; deliberately held at the 3.8 behavior (oxc-project/oxc#25841).
// The first member diverges at printWidth 80 (flat next line fits), the second
// at 100; at the respectively other width each is inline (80) or fully
// expanded (100) in both formatters.
interface Props {
  autoSelect?: "first" | "last" | "one" | ((item: OptionsItem[]) => OptionsItem) | false;
  wider?: "first" | "last" | "one" | "some" | "extra" | ((item: OptionsItem[]) => OptionsItem) | false;
}

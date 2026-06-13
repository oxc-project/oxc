// https://github.com/oxc-project/oxc/pull/22684
// `await` is reserved in a module, so this is a (correct) error. The point of
// this test is that the decorated `export default` must be recorded only once:
// it must NOT also report a spurious "A module cannot have multiple default
// exports". The `await` identifier is what previously triggered the reparse
// that double-recorded the export.
@foo
export default class C {
  x = await + 1;
}

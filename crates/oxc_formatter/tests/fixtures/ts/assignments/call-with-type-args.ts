// A type-argument call with one short argument breaks after the `=`
// (BreakAfterOperator), never inside the argument list.
// issue #17804
const fooRef =
  useRef<Record<string, LazyFooThingFD<T, TError> | null | undefined>>(cache);
const requestTrie =
  TernarySearchTree.forPaths<IRecursiveWatchRequest>(!isLinux);
// issue #18927: same rule for a class property (deeper indent)
class A {
  readonly customHeaderTemplate =
    viewChild.required<TemplateRef<{ total: number }>>("customHeader");
}

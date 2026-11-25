const arguments = [];

declare function ambient(value: typeof arguments): void;

function overload(value: typeof arguments): void;
function overload(value: any) {}

type FunctionType = (value: typeof arguments) => void;

interface Signatures {
  method(value: typeof arguments): void;
}

namespace Namespace {
  export function inner(value = arguments) {
    arguments;
  }
}

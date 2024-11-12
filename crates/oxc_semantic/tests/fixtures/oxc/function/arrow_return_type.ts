export type T = void;

((): T => { // Return type `T` resolves to top level `T`, not `T` in function body.
  type T = string;
});

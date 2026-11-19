// An assignment's class prints one pair around a superclass with parens of its own: DIVERGENCES.md#class-extends-single-parens.
DynamicErrorBoundary = class extends (
  React.Component<DynamicErrorBoundaryProps, DynamicErrorBoundaryState>
) {};
DynamicErrorBoundary = class extends (
  React.Component<DynamicErrorBoundaryProps, DynamicErrorBoundaryState>
)<DynamicErrorBoundaryTypeArgument> {};
DynamicErrorBoundary = class extends (
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa || bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
) {};
DynamicErrorBoundary = class extends (
  aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa = bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
) {};
DynamicErrorBoundary = class extends (
  someobjectsomepropertysomeotherproperty.someotherpropertyyyyyyyyy.SomeClass
) {};
a = class extends (Foo<Bar>) {};
a = class extends (B<T>)<U> {};
a = class extends (b || c) {};
a = class extends (b, c) {};
a = class extends /* c */ (Foo<T>) /* d */ {};
a = class extends /* prettier-ignore */ (b    ||    c) {};

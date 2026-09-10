// A template tag in statement position is a declaration: no terminator, and no `export
// default`, which is a second spelling of the same thing. We print the `as` and `satisfies`
// forms alike; the plugin omits the terminator for one and emits it for the other.
// See DIVERGENCES.md#template-tag-statement-terminator
export default <template>explicit</template>;

<template>bare</template>

<template>as</template> as Foo

<template>satisfies</template> satisfies Foo

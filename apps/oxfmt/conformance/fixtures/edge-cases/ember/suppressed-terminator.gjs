// The ignored `const` gains a terminator; the ignored bare tag does not.
// See DIVERGENCES.md#suppressed-declaration-terminator
// prettier-ignore
const a = <template>  a  </template>

// prettier-ignore
<template>  bare  </template>

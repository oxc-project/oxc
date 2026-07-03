let ignored = "x";

<template>
  {{! eslint-disable-next-line ember/template-no-let-reference }}
  <p>{{ignored}}</p>
</template>

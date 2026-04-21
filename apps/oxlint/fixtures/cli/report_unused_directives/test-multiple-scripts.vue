<template>
    <div>Hello World</div>
</template>

<script>
// eslint-disable-next-line no-debugger
console.log('regular script');

// eslint-disable-next-line no-console
debugger;
</script>

<script setup>
// eslint-disable
const unusedVariable1 = 42;

// eslint-disable-next-line no-unused-vars
const unusedVariable2 = 100;

function testFunction() {
    // eslint-disable-next-line no-console
    console.log('Inside test function');
}

// eslint-enable

// eslint-disable-next-line no-console, no-debugger
console.log('yes'); debugger;

// oxlint-disable-next-line no-debugger, no-for-loop
console.log("complete line");

testFunction();
</script>

<script setup lang="ts" generic="T extends Record<string, string>">
let foo: T;

// eslint-disable-next-line no-debugger
console.log('setup with generics');

// eslint-disable-next-line no-console
debugger;

// eslint-enable
</script>

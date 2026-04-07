<template>
    <div>Hello World</div>
</template>

<script>
// eslint-disable
const unusedVariable1 = 42;

// eslint-disable-next-line no-debugger
console.log('This is a test');

// eslint-enable

// eslint-disable-next-line no-console
debugger;

// eslint-disable-next-line no-unused-vars
const unusedVariable2 = 100;

function testFunction() {
    // eslint-disable-next-line no-console
    console.log('Inside test function');
}

// eslint-disable-next-line no-console, no-debugger
console.log('yes'); debugger;

// eslint-disable-next-line no-console, no-debugger
console.log('no');

// oxlint-disable-next-line no-debugger, no-for-loop
console.log("complete line");

testFunction();

// eslint-enable
</script>

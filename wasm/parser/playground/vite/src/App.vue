<script setup lang="ts">
import { parseSync } from "@oxc-parser/wasm";

const sampleSourceCode = `
    function myFn(){
        console.log("Here")
    }
`
const { program } = parseSync(sampleSourceCode)
</script>

<template>
  <div>
    Parsed Program:

    <pre>
      {{ program }}
    </pre>
  </div>
</template>

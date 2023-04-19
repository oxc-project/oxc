<script lang="ts">
  import CodeMirror, { basicSetup } from "../components/CodeMirror.svelte";
  import init, { Oxc } from "@oxc/wasm-web";

  let oxc;
  let doc = '';
  let docStore;
  let result;
  let ast

  function changeHandler({ detail: { tr } }) {
    console.log("change", tr.changes.toJSON());
  }

  async function main() {
    // This must be called before everything else
    await init();

    oxc = new Oxc();

    docStore.subscribe((content) => {
      let ret = oxc.parse(content);
      if (typeof ret === "string") {
        ast = "";
        result = ret
      } else {
        ast = JSON.stringify(ret, null, 4);
        result = ''
      }
    });
  }

  main();
</script>

<style>
  :global(.codemirror) {
  }

</style>

<div class="flex flex-1 bg-dark-400 divide-x">
  <div class="w-200px">
    Controls
  </div>

  <div class="flex-1 flex flex-col divide-y">
    <CodeMirror
      doc={doc}
      bind:docStore={docStore}
      extensions={basicSetup}
      on:change={changeHandler}
      placeholder="Enter your code here"
    />

    <div class="h-200px">
      <textarea
        class="w-full bg-dark-400"
        rows="10"
        value={result}
      />
    </div>
  </div>

  <div class="flex-1 flex flex-col whitespace-pre h-screen overflow-y text-xs">
      { ast }
  </div>

</div>

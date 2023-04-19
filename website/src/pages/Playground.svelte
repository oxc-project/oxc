<script>
  import CodeMirror, { basicSetup } from "../components/CodeMirror.svelte";
  import init, { parse } from "@oxc/wasm-web";

  let store;
  let defaultDoc = "var a = 233\n{}";
  let parseResult;

  function changeHandler({ detail: { tr } }) {
    console.log("change", tr.changes.toJSON());
  }

  async function autoParse() {
    await init();

    store.subscribe((content) => {
      parseResult = parse(content);
      console.info(parseResult);
    });
  }

  autoParse();
</script>

<style>
  :global(.codemirror) {
    height: 100%;
    overflow: auto;
  }

  :global(.cm-editor) {
    position: relative;
    display: flex !important;
    box-sizing: border-box;
    flex-direction: column;
    height: 100%;
  }

  :global(.cm-gutters) {
    background-color: rgba(34, 34, 34, var(--tw-bg-opacity)) !important;
    border: 0 !important;
  }

  :global(.cm-activeLineGutter) {
    background-color: rgba(34, 34, 34, var(--tw-bg-opacity)) !important;
  }
</style>

<div class="flex flex-1 bg-dark-400 divide-x">
  <div class="w-200px">
    Controls
  </div>

  <div class="flex-1 flex flex-col divide-y">
    <CodeMirror
      doc={defaultDoc}
      bind:docStore={store}
      extensions={basicSetup}
      on:change={changeHandler}
    />

    <div class="h-200px">
      <textarea
        class="w-full bg-dark-400"
        rows="10"
        value={typeof parseResult === "string" ? parseResult : "Good Job!"}
      />
    </div>
  </div>

  <div class="flex-1 flex flex-col divide-y">
    AST
  </div>

</div>

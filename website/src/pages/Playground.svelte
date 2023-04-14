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

<div class="flex flex-1 bg-dark-400">
  <section class="flex-1 w-50%">
    <CodeMirror
      doc={defaultDoc}
      bind:docStore={store}
      extensions={basicSetup}
      on:change={changeHandler}
    />
  </section>
  <section class="flex-1 w-50%">
    <textarea
      class="w-full"
      rows="10"
      value={typeof parseResult === "string" ? parseResult : "Good Job!"}
    />
  </section>
</div>

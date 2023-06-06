import { basicSetup } from "codemirror";
import { EditorView, keymap } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";

import initWasm, { Oxc, OxcOptions, OxcMinifierOptions } from "@oxc/wasm-web";

const placeholderText = `
import React, { useEffect, useRef } from 'react'

const DummyComponent:React.FC = () => {

  const ref = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (ref.current) ref.current.focus()
  }, [])

  return (
      <div>{Boolean(ref.current) ?? (
        <input type="text" ref={ref} />
      )}
      </div>
  )

}

export default DummyComponent
`.trim();

class Playground {
  oxc;
  options;
  editor;
  viewer;
  currentView = "ast"; // "ast" | "hir" | "format" | "minify"

  constructor() {
    this.editor = this.initEditor();
    this.viewer = this.initViewer();
  }

  initOxc() {
    this.oxc = new Oxc();
    this.options = new OxcOptions();
    this.oxc.setOptions(this.options);
    // This will trigger `stateListener`.
    this.updateEditorText(this.editor, placeholderText);
  }

  initEditor() {
    const stateListener = EditorView.updateListener.of((view) => {
      if (view.docChanged) {
        const sourceText = view.state.doc.toString();
        this.oxc.setSourceText(sourceText);
        this.run();
        this.updateView(this.currentView);
      }
    });

    const state = EditorState.create({
      doc: "Loading Wasm... (~400kb)",
      extensions: [
        basicSetup,
        keymap.of(vscodeKeymap),
        javascript(),
        githubDark,
        lintGutter(),
        stateListener,
        linter((_view) => {
          return this.oxc
            ? this.oxc.getDiagnostics().map((d) => ({
                from: d.start,
                to: d.end,
                severity: d.severity.toLowerCase(),
                message: d.message,
              }))
            : [];
        }),
      ],
    });

    return new EditorView({
      state,
      parent: document.querySelector("#editor"),
    });
  }

  initViewer() {
    return new EditorView({
      extensions: [
        javascript(),
        githubDark,
        EditorView.editable.of(false),
        EditorView.lineWrapping,
      ],
      parent: document.querySelector("#display"),
    });
  }

  run() {
    const start = new Date();
    this.oxc.run();
    const elapsed = new Date() - start;
    document.getElementById("duration").innerText = `${elapsed}ms`;
    this.updatePanel();
  }

  updatePanel() {
    const panel = document.getElementById("panel");
    panel.innerText = this.oxc
      .getDiagnostics()
      .map((d) => d.message)
      .join("\n");
    panel.scrollTop = panel.scrollHeight;
  }

  updateView(view) {
    this.currentView = view;
    let text;
    switch (this.currentView) {
      case "ast":
        text = JSON.stringify(this.oxc.getAst(), null, 2);
        break;
      case "hir":
        text = JSON.stringify(this.oxc.getHir(), null, 2);
        break;
      case "format":
        text = this.oxc.getFormattedText();
        break;
      case "minify":
        text = this.oxc.getMinifiedText();
        break;
    }
    this.updateEditorText(this.viewer, text);
  }

  updateEditorText(instance, text) {
    const transaction = instance.state.update({
      changes: { from: 0, to: instance.state.doc.length, insert: text },
    });
    instance.dispatch(transaction);
  }
}

async function main() {
  const playground = new Playground();

  await initWasm();

  playground.initOxc();

  window.setTimeout(function () {
    playground.editor.focus();
  }, 0);

  document.getElementById("ast").onclick = () => {
    playground.updateView("ast");
  };

  document.getElementById("hir").onclick = () => {
    playground.updateView("hir");
  };

  document.getElementById("format").onclick = () => {
    playground.updateView("format");
  };

  document.getElementById("minify").onclick = function () {
    playground.updateView("minify");
  };

  document.getElementById("mangle").onchange = function () {
    const checked = document.getElementById("mangle-checkbox").checked;
    const options = new OxcOptions();
    const minifiedOptions = new OxcMinifierOptions();
    minifiedOptions.mangle = checked;
    options.minifier = minifiedOptions;
    playground.oxc.setOptions(options);
    playground.run();
    playground.updateView("minify");
  };
}

main();

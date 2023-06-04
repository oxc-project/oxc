import { basicSetup } from "codemirror";
import { EditorView, keymap, showPanel } from "@codemirror/view";
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

  constructor() {
    this.oxc = new Oxc();
    this.options = new OxcOptions();
    this.oxc.setOptions(this.options);
    this.oxc.setSourceText(placeholderText);
    this.oxc.run();
    this.editor = this.initEditor();
    this.viewer = this.initViewer();
  }

  initEditor() {
    const stateListener = EditorView.updateListener.of((view) => {
      if (view.docChanged) {
        const sourceText = view.state.doc.toString();
        this.oxc.setSourceText(sourceText);
        this.oxc.run();
        this.updateViewer(this.getAst());
      }
    });

    const state = EditorState.create({
      doc: this.oxc.getSourceText(),
      extensions: [
        basicSetup,
        keymap.of(vscodeKeymap),
        javascript(),
        githubDark,
        lintGutter(),
        showPanel.of(this.getConsolePanel.bind(this)),
        stateListener,
        linter((_view) => {
          return this.oxc.getDiagnostics().map((d) => ({
            from: d.start,
            to: d.end,
            severity: d.severity.toLowerCase(),
            message: d.message,
          }));
        }),
      ],
    });

    return new EditorView({
      state,
      parent: document.querySelector("#app"),
    });
  }

  initViewer() {
    return new EditorView({
      doc: this.getAst(),
      extensions: [
        basicSetup,
        javascript(),
        githubDark,
        EditorView.editable.of(false),
        EditorView.lineWrapping,
      ],
      parent: document.querySelector("#display"),
    });
  }

  getAst() {
    return JSON.stringify(this.oxc.getAst(), null, 2);
  }

  getHir() {
    return JSON.stringify(this.oxc.getHir(), null, 2);
  }

  getFormattedText() {
    return this.oxc.getFormattedText();
  }

  getMinifiedText() {
    return this.oxc.getMinifiedText();
  }

  getConsole(_doc) {
    return this.oxc
      .getDiagnostics()
      .map((d) => d.message)
      .join("\n");
  }

  getConsolePanel(view) {
    const that = this;
    const dom = document.createElement("div");
    dom.textContent = that.getConsole(view.state.doc);
    return {
      dom,
      update(update) {
        if (update.docChanged) {
          dom.textContent = that.getConsole(update.state.doc);
          dom.scrollTop = dom.scrollHeight;
        }
      },
    };
  }

  updateViewer(text) {
    const transaction = this.viewer.state.update({
      changes: { from: 0, to: this.viewer.state.doc.length, insert: text },
    });
    this.viewer.dispatch(transaction);
  }
}

async function main() {
  await initWasm();

  const playground = new Playground();

  window.setTimeout(function () {
    playground.editor.focus();
  });

  document.getElementById("ast").onclick = function () {
    playground.updateViewer(playground.getAst());
  };

  document.getElementById("hir").onclick = function () {
    playground.updateViewer(playground.getHir());
  };

  document.getElementById("format").onclick = function () {
    playground.updateViewer(playground.getFormattedText());
  };

  document.getElementById("minify").onclick = function () {
    playground.updateViewer(playground.getMinifiedText());
  };

  document.getElementById("mangle").onchange = function () {
    const checked = document.getElementById("mangle-checkbox").checked;
    const options = new OxcOptions();
    const minifiedOptions = new OxcMinifierOptions();
    minifiedOptions.mangle = checked;
    options.minifier = minifiedOptions;
    playground.oxc.setOptions(options);
    playground.oxc.run();
    playground.updateViewer(playground.getMinifiedText());
  };
}

main();

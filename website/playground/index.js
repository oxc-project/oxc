import { basicSetup } from "codemirror";
import { EditorView, keymap, Decoration } from "@codemirror/view";
import {
  EditorState,
  StateEffect,
  StateField,
  EditorSelection,
  Compartment,
  RangeSet,
} from "@codemirror/state";
import { javascript, javascriptLanguage } from "@codemirror/lang-javascript";
import { json, jsonLanguage } from "@codemirror/lang-json";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";
import { language, syntaxTree } from "@codemirror/language";
import throttle from "lodash.throttle";

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
  languageConf;
  urlParams;

  constructor() {
    this.languageConf = new Compartment();
    this.urlParams = new URLParams();
    this.editor = this.initEditor();
    this.viewer = this.initViewer();
  }

  initOxc() {
    this.oxc = new Oxc();
    this.options = new OxcOptions();
    this.oxc.setOptions(this.options);
    // This will trigger `stateListener`.
    this.updateEditorText(this.editor, this.urlParams.code || placeholderText);
  }

  initEditor() {
    const stateListener = EditorView.updateListener.of((view) => {
      if (view.docChanged) {
        const sourceText = view.state.doc.toString();
        this.urlParams.updateCode(sourceText);
        this.oxc.setSourceText(sourceText);
        this.run();
        this.updateView(this.currentView);
      }
    });

    const state = EditorState.create({
      doc: "Loading Wasm... (~400kb)",
      extensions: [
        basicSetup,
        EditorView.lineWrapping,
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
        githubDark,
        EditorView.editable.of(false),
        EditorView.lineWrapping,
        this.languageConf.of(javascript()),
        // Change language according to the current view
        EditorState.transactionExtender.of((tr) => {
          if (!tr.docChanged) return null;
          const currentLanguage = tr.startState.facet(language);
          let newLanguage;
          switch (this.currentLanguage()) {
            case "json":
              if (currentLanguage == jsonLanguage) return null;
              newLanguage = json();
              break;
            case "javascript":
              if (currentLanguage == javascriptLanguage) return null;
              newLanguage = javascript();
              break;
          }
          return {
            effects: this.languageConf.reconfigure(newLanguage),
          };
        }),
        EditorView.domEventHandlers({
          mouseover: this.highlightEditorFromViewer.bind(this),
        }),
      ],
      parent: document.querySelector("#viewer"),
    });
  }

  run() {
    const start = new Date();
    this.oxc.run();
    const elapsed = new Date() - start;
    document.getElementById("duration").innerText = `${elapsed}ms`;
    this.updatePanel();
  }

  currentLanguage() {
    switch (this.currentView) {
      case "ast":
      case "hir":
        return "json";
      default:
        return "javascript";
    }
  }

  updatePanel() {
    const panel = document.getElementById("panel");
    panel.innerText = this.oxc
      .getDiagnostics()
      .map((d) => d.message)
      .join("\n\n");
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

  highlightEditorRange(view, range) {
    if (range.from === 0 && range.to === 0) {
      return
    }
    const addHighlight = StateEffect.define({
      map: ({ from, to }, change) => ({
        from: change.mapPos(from),
        to: change.mapPos(to),
      }),
    });
    const highlightField = StateField.define({
      create() {
        return Decoration.none;
      },
      update(highlights, tr) {
        highlights = RangeSet.empty;
        for (let e of tr.effects) {
          if (e.is(addHighlight)) {
            highlights = highlights.update({
              add: [Playground.highlightMark.range(e.value.from, e.value.to)],
            });
          }
        }
        return highlights;
      },
      provide: (f) => EditorView.decorations.from(f),
    });
    const effects = [addHighlight.of(range)];
    if (!view.state.field(highlightField, false)) {
      effects.push(
        StateEffect.appendConfig.of([highlightField, Playground.highlightTheme])
      );
    }
    view.dispatch({ effects });
  }

  getTextFromView(view, from, to) {
    return view.state.doc.sliceString(from, to);
  }

  static highlightMark = Decoration.mark({ class: "cm-highlight" });
  static highlightTheme = EditorView.baseTheme({
    ".cm-highlight": { background: "black" },
  });

  // Highlight the editor by searching for `start` and `end` values.
  highlightEditorFromViewer(e, view) {
    if (this.currentLanguage() != "json") {
      return;
    }
    const pos = view.posAtCoords(e);
    const tree = syntaxTree(view.state);
    let cursor = tree.cursorAt(pos);
    // Go up and find the `type` key
    while (true) {
      if (view.state.doc.sliceString(cursor.from, cursor.to) == '"type"') {
        break;
      }
      if (!cursor.prev()) {
        break;
      }
    }
    // Go down and find the `start` and `end` keys
    let start, end;
    while (true) {
      if (
        !start &&
        this.getTextFromView(view, cursor.from, cursor.to) == '"start"'
      ) {
        cursor.next();
        start = this.getTextFromView(view, cursor.from, cursor.to);
      }
      if (
        !end &&
        this.getTextFromView(view, cursor.from, cursor.to) == '"end"'
      ) {
        cursor.next();
        end = this.getTextFromView(view, cursor.from, cursor.to);
      }
      if (start && end) {
        break;
      }
      if (!cursor.next()) {
        break;
      }
    }
    this.highlightEditorRange(
      this.editor,
      EditorSelection.range(Number(start), Number(end))
    );
  }
}

// Code copied from Rome
// <https://github.com/rome/tools/blob/665bb9d810b4ebf4ea82b72df20ad79b8fa3a3d0/website/src/playground/utils.ts#L141-L181>
class URLParams {
  // Safari/Webkit/JSC/whatever only allows setting a URL 50 times within 30 seconds
  // set our maximum update frequency just under that to avoid any chance of hitting it
  static URL_UPDATE_THROTTLE = 30000 / 40;

  params;
  code;

  constructor() {
    this.params = new URLSearchParams(window.location.search);
    this.code = this.params.has("code")
      ? this.decodeCode(this.params.get("code"))
      : "";
    console.log(this.code);
  }

  updateCode = throttle(
    (code) => {
      this.code = this.encodeCode(code);
      this.params.set("code", this.code);
      const url = `${window.location.protocol}//${window.location.host}${
        window.location.pathname
      }?${this.params.toString()}`;
      window.history.replaceState({ path: url }, "", url);
    },
    URLParams.URL_UPDATE_THROTTLE,
    { trailing: true }
  );

  // See https://developer.mozilla.org/en-US/docs/Web/API/btoa#unicode_strings
  encodeCode(code) {
    return btoa(this.toBinary(code));
  }

  decodeCode(encoded) {
    try {
      return this.fromBinary(atob(encoded));
    } catch {
      return encoded;
    }
  }

  // convert a Unicode string to a string in which
  // each 16-bit unit occupies only one byte
  toBinary(input) {
    const codeUnits = new Uint16Array(input.length);
    for (let i = 0; i < codeUnits.length; i++) {
      codeUnits[i] = input.charCodeAt(i);
    }

    const charCodes = new Uint8Array(codeUnits.buffer);
    let result = "";
    for (let i = 0; i < charCodes.byteLength; i++) {
      result += String.fromCharCode(charCodes[i]);
    }
    return result;
  }

  fromBinary(binary) {
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < bytes.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    const charCodes = new Uint16Array(bytes.buffer);
    let result = "";
    for (let i = 0; i < charCodes.length; i++) {
      result += String.fromCharCode(charCodes[i]);
    }
    return result;
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

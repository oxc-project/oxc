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
import { rust, rustLanguage } from "@codemirror/lang-rust";
import { json, jsonLanguage } from "@codemirror/lang-json";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";
import { language, syntaxTree } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import throttle from "lodash.throttle";

import initWasm, {
  Oxc,
  OxcRunOptions,
  OxcParserOptions,
  OxcLinterOptions,
  OxcMinifierOptions,
  OxcFormatterOptions,
  OxcTypeCheckingOptions,
} from "@oxc/wasm-web";

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

  runOptions;
  parserOptions;
  formatterOptions;
  linterOptions;
  minifierOptions;

  editor;
  viewer;
  currentView = "ast"; // "ast" | "hir" | "format" | "minify" | "ir"
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
    this.runOptions = new OxcRunOptions();
    this.parserOptions = new OxcParserOptions();
    this.formatterOptions = new OxcFormatterOptions();
    this.linterOptions = new OxcLinterOptions();
    this.minifierOptions = new OxcMinifierOptions();
    this.typeCheckOptions = new OxcTypeCheckingOptions();
    // This will trigger `stateListener`.
    this.updateEditorText(this.editor, this.urlParams.code || placeholderText);
  }

  initEditor() {
    const stateListener = EditorView.updateListener.of((view) => {
      if (view.docChanged) {
        const sourceText = view.state.doc.toString();
        this.urlParams.updateCode(sourceText);
        this.oxc.sourceText = sourceText;
        this.updateView();
      }
    });

    const state = EditorState.create({
      extensions: [
        basicSetup,
        EditorView.lineWrapping,
        keymap.of(vscodeKeymap),
        javascript(),
        githubDark,
        lintGutter(),
        stateListener,
        autocompletion(),
        linter(
          () => {
            const diagnostics = this.oxc
              ? this.oxc.getDiagnostics().map((d) => ({
                  from: d.start,
                  to: d.end,
                  severity: d.severity.toLowerCase(),
                  message: d.message,
                }))
              : [];
            this.updatePanel(diagnostics);
            return diagnostics;
          },
          { delay: 0 }
        ),
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
            case "rust":
              if (currentLanguage == rustLanguage) return null;
              newLanguage = rust();
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
    this.oxc.run(
      this.runOptions,
      this.parserOptions,
      this.linterOptions,
      this.formatterOptions,
      this.minifierOptions,
      this.typeCheckOptions
    );
    const elapsed = new Date() - start;
    document.getElementById("duration").innerText = `${elapsed}ms`;
  }

  currentLanguage() {
    switch (this.currentView) {
      case "ir":
        return "rust";
      case "ast":
      case "hir":
        return "json";
      default:
        return "javascript";
    }
  }

  updatePanel(diagnostics) {
    const panel = document.getElementById("panel");
    panel.innerText = diagnostics
      .map((d) => {
        const emoji = {
          error: "❗",
          warning: "⚠️",
          advice: "ℹ️",
        }[d.severity.toLowerCase()];
        return `${emoji} ${d.message}`;
      })
      .join("\n\n");
    panel.scrollTop = panel.scrollHeight;
  }

  updateView(view) {
    view = view || this.currentView;
    this.currentView = view;

    document.getElementById("mangle").style.visibility = "hidden";
    document.getElementById("ir-copy").style.display = "none";
    this.runOptions.format = false;
    this.runOptions.hir = false;
    this.runOptions.minify = false;

    let text;
    switch (this.currentView) {
      case "ast":
        this.run();
        text = JSON.stringify(this.oxc.ast, null, 2);
        break;
      case "hir":
        this.runOptions.hir = true;
        this.run();
        text = JSON.stringify(this.oxc.hir, null, 2);
        break;
      case "ir":
        document.getElementById("ir-copy").style.display = "inline";
        this.runOptions.ir = true;
        this.run();
        text = this.oxc.ir;
        break;
      case "format":
        this.runOptions.format = true;
        this.run();
        text = this.oxc.formattedText;
        break;
      case "minify":
        document.getElementById("mangle").style.visibility = "visible";
        this.runOptions.minify = true;
        this.run();
        text = this.oxc.minifiedText;
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
      return;
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
    ".cm-highlight": { background: "#3392FF44" },
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

  document.getElementById("loading").remove();

  document.getElementById("ast").onclick = () => {
    playground.updateView("ast");
  };

  document.getElementById("hir").onclick = () => {
    playground.updateView("hir");
  };

  document.getElementById("ir").onclick = () => {
    playground.updateView("ir");
  };

  document.getElementById("ir-copy").onclick = () => {
    navigator.clipboard.writeText(playground.oxc.ir);
  };

  // document.getElementById("format").onclick = () => {
  // playground.updateView("format");
  // };

  document.getElementById("minify").onclick = function () {
    playground.updateView("minify");
  };

  document.getElementById("syntax").onchange = function () {
    const checked = document.getElementById("syntax-checkbox").checked;
    playground.runOptions.syntax = checked;
    // Need to repaint the editor to clear the rendered linter diagnostics
    const sourceText = playground.oxc.sourceText;
    playground.updateEditorText(playground.editor, "");
    playground.updateView();
    playground.updateEditorText(playground.editor, sourceText);
  };

  document.getElementById("lint").onchange = function () {
    const checked = document.getElementById("lint-checkbox").checked;
    playground.runOptions.lint = checked;
    // Need to repaint the editor to clear the rendered linter diagnostics
    const sourceText = playground.oxc.sourceText;
    playground.updateEditorText(playground.editor, "");
    playground.updateView();
    playground.updateEditorText(playground.editor, sourceText);
  };

  document.getElementById("mangle").onchange = function () {
    const checked = document.getElementById("mangle-checkbox").checked;
    playground.minifierOptions.mangle = checked;
    playground.updateView("minify");
  };

  document.getElementById("type-check").onchange = function () {
    const checked = document.getElementById("type-check-checkbox").checked;
    playground.runOptions.type_check = checked;
    playground.updateView();
  };
}

main();

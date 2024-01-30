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
import  { convertToUtf8, getStartAndEnd } from './editor.js'
import  { findMostInnerNodeForPosition } from './traverseJson.js'
import { parser } from '@lezer/json'
import { javascript, javascriptLanguage } from "@codemirror/lang-javascript";
import { rust, rustLanguage } from "@codemirror/lang-rust";
import { json, jsonLanguage } from "@codemirror/lang-json";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";
import { language, syntaxTree } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import { indentWithTab, deleteLine } from "@codemirror/commands";
import throttle from "lodash.throttle";

// lzma is a very old library, it writes to window when built in production with vite.
import { LZMA } from 'lzma/src/lzma_worker.js';
const GLOBAL_LZMA = LZMA || window.LZMA;

import initWasm, {
  Oxc,
  OxcRunOptions,
  OxcParserOptions,
  OxcLinterOptions,
  OxcMinifierOptions,
  OxcCodegenOptions,
} from "@oxc/oxc_wasm";
import { getSymbolAndReferencesSpan, renderSymbols } from "./symbols.js";

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

const STORAGE_KEY_CODE = "playground.code";
const ACTIVE_TAB_STORAGE_KEY_CODE = "playground.activeTab";

const getStringFromStorage = (whatToGet) => {
  try {
    return localStorage.getItem(whatToGet);
  } catch (_e) {
    return "";
  }
};

const setStringToStorage = (whatToSet, value) => {
  try {
    localStorage.setItem(whatToSet, value);
  } catch (_e) {
    return;
  }
};

const tabBtnsBindClick = (callback) => {
  const buttons = document.querySelectorAll('.header.controls button');
  buttons.forEach(btn => {
    btn.onclick = (e) => {
      callback(e?.target?.id);
    };
  });
}

const switchActiveTab = (tab) => {
  const buttons = document.querySelectorAll('.header.controls button');
  let targetBtn = null;
  buttons.forEach(btn => {
    btn.classList.remove('active')
    if (tab === btn.id) {
      targetBtn = btn;
    }
  });
  targetBtn?.classList.add('active');
}

const initActiveTab = () => {
  const activeTab = getStringFromStorage(ACTIVE_TAB_STORAGE_KEY_CODE);
  if (!activeTab) {
    return;
  }
  const btn = document.getElementById(activeTab);
  if (!btn?.classList) {
    return;
  }
  btn.classList.add('active');
}

class Playground {
  oxc;
  sourceTextUtf8 // source text in Uint8Array, for converting from utf8 to utf16 span

  runOptions;
  parserOptions;
  codegenOptions;
  linterOptions;
  minifierOptions;

  editor;
  viewer;
  currentView = "ast"; // "ast" | "format" | "minify" | "ir"
  languageConf;
  urlParams;
  viewerIsEditableConf;

  constructor() {
    this.languageConf = new Compartment();
    this.urlParams = new URLParams();
    this.viewerIsEditableConf = new Compartment();
    this.linterConf = new Compartment();
    this.editor = this.initEditor();
    this.viewer = this.initViewer();
    this.currentView = getStringFromStorage(ACTIVE_TAB_STORAGE_KEY_CODE) || "ast";
  }

  initOxc() {
    this.oxc = new Oxc();
    this.runOptions = new OxcRunOptions();
    this.parserOptions = new OxcParserOptions();
    this.codegenOptions = new OxcCodegenOptions();
    this.linterOptions = new OxcLinterOptions();
    this.minifierOptions = new OxcMinifierOptions();

    this.runOptions.syntax = true;
    this.runOptions.lint = true;

    this.runOxc(this.editor.state.doc.toString());
    this.editor.dispatch({ effects: this.linterConf.reconfigure(this.linter()) });
  }

  linter() {
    return linter(() => this.updateDiagnostics(), { delay: 0 })
  }

  runOxc(text) {
    const sourceText = text;
    this.urlParams.updateCode(sourceText);
    this.oxc.sourceText = sourceText;
    this.sourceTextUtf8 = new TextEncoder().encode(sourceText);
    this.updateView();
  }

  initEditor() {
    const stateListener = EditorView.updateListener.of((view) => {
      if (view.docChanged) {
        this.runOxc(view.state.doc.toString());
        return;
      }
      if (!view.docChanged && view.selectionSet && this.currentView === 'ast') {
        let ranges = view.state.selection.ranges;
        if (ranges.length === 1 && ranges[0].empty) {
          this.editorRange = view.state.selection.ranges
          let {from} = this.editorRange[0]
          let viewerText = this.viewer.state.doc.toString();
          let ast = parser.parse(viewerText)
          let root = ast.cursor().node;
          let targetNode = findMostInnerNodeForPosition(root.node, from, viewerText)
          if (!targetNode?.from) {
            return;
          }
          this.viewer.dispatch({
            selection: EditorSelection.single(targetNode.to, targetNode.from),
            scrollIntoView: true,
          })
        }
      }
    });

    const state = EditorState.create({
      extensions: [
        basicSetup,
        EditorView.lineWrapping,
        keymap.of([
          ...vscodeKeymap,
          indentWithTab,
          {
            key: "Delete",
            shift: deleteLine,
          },
        ]),
        javascript(),
        githubDark,
        lintGutter(),
        stateListener,
        autocompletion(),
        this.linterConf.of(this.linter()),
      ],
      doc: this.urlParams.code || placeholderText,
    });

    return new EditorView({
      state,
      parent: document.querySelector("#editor"),
    });
  }

  updateDiagnostics() {
    const diagnostics = (this.oxc ? this.oxc.getDiagnostics() : []).map((d) => ({
      from: convertToUtf8(this.sourceTextUtf8, d.start),
      to: convertToUtf8(this.sourceTextUtf8, d.end),
      severity: d.severity.toLowerCase(),
      message: d.message,
    }));
    this.updatePanel(diagnostics);
    return diagnostics;
  }

  initViewer() {
    // scroll selection into the middle https://discuss.codemirror.net/t/cm6-scroll-to-middle/2924/2
    const viewStateListener = EditorView.updateListener.of((update) => {
      if (update.transactions.some(tr => tr.scrollIntoView)) {
        let view = update.view
        // (Sync with other DOM read/write phases for efficiency)
        view.requestMeasure({
          read() {
            return {
              cursor: view.coordsAtPos(view.state.selection.main.head),
              scroller: view.scrollDOM.getBoundingClientRect()
            }
          },
          write({cursor, scroller}) {
            if (cursor) {
              let curMid = (cursor.top + cursor.bottom) / 2
              let eltMid = (scroller.top + scroller.bottom) / 2
              if (Math.abs(curMid - eltMid) > 5)
              view.scrollDOM.scrollTop += curMid - eltMid
            }
          }
        })
      }
    });
    return new EditorView({
      extensions: [
        linter(
          () => {
            try {
              this.runQuery();
            } finally {
              return [];
            }
          },
          { delay: 0 }
        ),
        viewStateListener,
        basicSetup,
        keymap.of([
          ...vscodeKeymap,
          indentWithTab,
          {
            key: "Delete",
            shift: deleteLine,
          },
        ]),
        githubDark,
        EditorState.transactionExtender.of((tr) => {
          if (!tr.docChanged) return null;
          let ext = EditorView.editable.of(false);
          return {
            effects: this.viewerIsEditableConf.reconfigure(ext),
          };
        }),
        this.viewerIsEditableConf.of(EditorView.editable.of(false)),
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
        autocompletion(),
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
      this.codegenOptions,
      this.minifierOptions,
    );
    const elapsed = new Date() - start;
    document.getElementById("duration").innerText = `${elapsed}ms`;
  }

  currentLanguage() {
    switch (this.currentView) {
      case "ir":
        return "rust";
      case "ast":
      case "symbol":
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

    document.getElementById("ir-copy").style.display = "none";
    document.getElementById("duration").style.display = "inline";
    document.getElementById("panel").style.display = "inline";
    this.runOptions.format = false;
    this.runOptions.minify = false;

    let text;
    switch (this.currentView) {
      case "ast":
        this.run();
        text = JSON.stringify(this.oxc.ast, null, 2);
        break;
      case "scope":
        this.runOptions.scope = true;
        this.run();
        this.runOptions.scope = false
        text = this.oxc.scopeText;
        break;
      case "symbol":
        this.runOptions.symbol = true;
        this.run();
        this.runOptions.symbol = false
        text = renderSymbols(this.oxc.symbols)
        break;
      case "codegen":
        this.run();
        text = this.oxc.codegenText;
        break;
      case "ir":
        document.getElementById("ir-copy").style.display = "inline";
        this.runOptions.ir = true;
        this.run();
        text = this.oxc.ir;
        break;
      case "prettier-ir":
        this.runOptions.prettier_ir = true;
        this.run();
        text = this.oxc.prettierIrText;
        break;
      case "prettier":
        this.runOptions.prettier_format = true;
        this.run();
        text = this.oxc.prettierFormattedText;
        break;
      case "format":
        this.runOptions.format = true;
        this.run();
        text = this.oxc.formattedText;
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
    let ranges = Array.isArray(range) ? range : [range];
    ranges = ranges.filter((range) => range.from !== 0 || range.to !== 0);
    if (ranges.length === 0) {
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
    const effects = ranges.map((range) => addHighlight.of(range));
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
    if (this.currentView === 'symbol') {
      const pos = view.posAtCoords(e);
      const tree = syntaxTree(view.state);
      let cursor = tree.cursorAt(pos);
      let [start, end] = getStartAndEnd.call(this, view, cursor)
      // if we didn't find a start or an end, return early
      if (start == undefined || end == undefined) return;
      let ranges = getSymbolAndReferencesSpan(start, end)
      this.highlightEditorRange(
        this.editor,
        ranges.map(range => EditorSelection.range(convertToUtf8(this.sourceTextUtf8, range.start), convertToUtf8(this.sourceTextUtf8, range.end)))
      );
    }
    else if (this.currentView === "ast") {
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
      let [start, end] = getStartAndEnd.call(this, view, cursor)
      // if we didn't find a start or an end, return early
      if (start == undefined || end == undefined) return;

      start = convertToUtf8(this.sourceTextUtf8, start)
      end = convertToUtf8(this.sourceTextUtf8, end)

      this.highlightEditorRange(
        this.editor,
        EditorSelection.range(start, end)
      );
    }
  }
}

// Code partly copied from Rome
// <https://github.com/rome/tools/blob/665bb9d810b4ebf4ea82b72df20ad79b8fa3a3d0/website/src/playground/utils.ts#L141-L181>
class URLParams {
  // Safari/Webkit/JSC/whatever only allows setting a URL 50 times within 30 seconds
  // set our maximum update frequency just under that to avoid any chance of hitting it
  static URL_UPDATE_THROTTLE = 30000 / 40;

  params;
  code;

  constructor() {
    this.params = new URLSearchParams(window.location.search);
    this.code = this.tryReadCode(this.params);
  }

  tryReadCode(params) {
    try {
      if (params.has("code")) {
        return this.decodeCode(params.get("code"));
      }
      return getStringFromStorage(STORAGE_KEY_CODE);
    } catch(e) {
      console.error(e);
      return ''
    }

  }

  updateCode = throttle(
    (code) => {
      this.code = this.encodeCode(code);
      this.params.set("code", this.code);
      const url = `${window.location.protocol}//${window.location.host}${
        window.location.pathname
      }?${this.params.toString()}`;
      window.history.replaceState({ path: url }, "", url);
      setStringToStorage(STORAGE_KEY_CODE, code);
    },
    URLParams.URL_UPDATE_THROTTLE,
    { trailing: true }
  );

  encodeCode(code) {
    const lzma = GLOBAL_LZMA.compress(code);
    return this.LZMABufferToBase64(lzma);
  }

  decodeCode(encoded) {
    const compressed = this.base64ToLZMABuffer(encoded);
    return GLOBAL_LZMA.decompress(compressed);
  }

  // https://developer.mozilla.org/en-US/docs/Glossary/Base64#the_unicode_problem
  // btoa is safe here, because we manually construct a string of code points
  // which are guaranteed to be one-byte chars
  // the 128 offset is to compensate for LZMA's -128 to 127 range
  LZMABufferToBase64 = (buffer) => btoa(Array.from(buffer, (x) => String.fromCodePoint(x + 128)).join(""));
  base64ToLZMABuffer = (base64) => Uint8Array.from(atob(base64), (m) => m.codePointAt(0) - 128);
}

async function main() {
  const playground = new Playground();

  await initWasm();

  playground.initOxc();

  window.setTimeout(function () {
    playground.editor.focus();
  }, 0);

  document.getElementById("loading").remove();

  addHorizontalResize()

  initActiveTab();
  tabBtnsBindClick((tab) => {
    if (tab === 'ir-copy') {
      navigator.clipboard.writeText(playground.oxc.ir);
      return;
    }
    playground.updateView(tab);
    switchActiveTab(tab);
    setStringToStorage(ACTIVE_TAB_STORAGE_KEY_CODE, tab);
  });

  // document.getElementById("format").onclick = () => {
  // playground.updateView("format");
  // };

  document.getElementById("transform").onchange = function () {
    const checked = document.getElementById("transform-checkbox").checked;
    playground.runOptions.transform = checked;
    playground.updateView("codegen");
  };

  document.getElementById("whitespace").onchange = function () {
    const checked = document.getElementById("whitespace-checkbox").checked;
    playground.minifierOptions.whitespace = checked;
    playground.updateView("codegen");
  };

  document.getElementById("mangle").onchange = function () {
    const checked = document.getElementById("mangle-checkbox").checked;
    playground.minifierOptions.mangle = checked;
    playground.updateView("codegen");
  };

  document.getElementById("compress").onchange = function () {
    const checked = document.getElementById("compress-checkbox").checked;
    playground.minifierOptions.compress = checked;
    playground.updateView("codegen");
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
}

// port from https://github.com/fkling/astexplorer/blob/541552fe45885c225fbb67d54dc4c6d6107b65b5/website/src/components/SplitPane.js#L26-L55
function addHorizontalResize() {
  const container = document.getElementById("container");
  const left = document.getElementById("left");
  const divider = document.getElementById("divider");

  divider.addEventListener("mousedown", function (event) {
    // This is needed to prevent text selection in Safari
    event.preventDefault();
    const offset = container.offsetLeft;
    const size = container.offsetWidth;
    const setStyle = (position) => {
      left.style.minWidth = left.style.maxWidth = position + '%'
    }
    globalThis.document.body.style.cursor = 'col-resize';

    const moveHandler = event => {
      event.preventDefault();
      const newPosition = ( event.pageX - offset) / size * 100;
      // Using 99% as the max value prevents the divider from disappearing
      const position = Math.min(Math.max(0, newPosition), 99);
      setStyle(position)
    };
    let upHandler = () => {
      document.removeEventListener('mousemove', moveHandler);
      document.removeEventListener('mouseup', upHandler);
      globalThis.document.body.style.cursor = '';
    };

    document.addEventListener('mousemove', moveHandler);
    document.addEventListener('mouseup', upHandler);
  })
}

main();

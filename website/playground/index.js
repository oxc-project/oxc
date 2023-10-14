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
import { graphql, graphqlLanguage } from "cm6-graphql";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";
import { language, syntaxTree } from "@codemirror/language";
import { autocompletion } from "@codemirror/autocomplete";
import { indentWithTab, deleteLine } from "@codemirror/commands";
import throttle from "lodash.throttle";
import { buildSchema } from "graphql";

// lzma is a very old library, it writes to window when built in production with vite.
import { LZMA } from 'lzma/src/lzma_worker.js';
const GLOBAL_LZMA = LZMA || window.LZMA;

import initWasm, {
  Oxc,
  OxcRunOptions,
  OxcParserOptions,
  OxcLinterOptions,
  OxcMinifierOptions,
  OxcFormatterOptions,
  OxcTypeCheckingOptions,
  graphql_schema_text,
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

const STORAGE_KEY_CODE = "playground.code";
const STORAGE_KEY_QUERY = "playground.query";
const STORAGE_KEY_QUERY_ARGUMENTS = "playground.query_arguments";

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

class Playground {
  oxc;
  sourceTextUtf8 // source text in Uint8Array, for converting from utf8 to utf16 span

  runOptions;
  parserOptions;
  formatterOptions;
  linterOptions;
  minifierOptions;

  editor;
  viewer;
  queryResultsViewer;
  currentView = "ast"; // "ast" | "format" | "minify" | "ir"
  languageConf;
  urlParams;
  viewerIsEditableConf;
  queryResultViewerIsEditableConf;
  showingQueryResultsOrArguments;

  constructor() {
    this.languageConf = new Compartment();
    this.urlParams = new URLParams();
    this.viewerIsEditableConf = new Compartment();
    this.queryResultViewerIsEditableConf = new Compartment();
    this.editor = this.initEditor();
    this.viewer = this.initViewer();
    this.queryResultsViewer = this.initQueryResultsViewer();
    this.showingQueryResultsOrArguments = "results";

  }

  initOxc() {
    this.oxc = new Oxc();
    this.runOptions = new OxcRunOptions();
    this.parserOptions = new OxcParserOptions();
    this.formatterOptions = new OxcFormatterOptions();
    this.linterOptions = new OxcLinterOptions();
    this.minifierOptions = new OxcMinifierOptions();
    this.typeCheckOptions = new OxcTypeCheckingOptions();

    this.runOptions.syntax = true;
    this.runOptions.lint = true;

    this.runOxc(this.editor.state.doc.toString());
    this.updateDiagnostics();
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
        linter(
          () => {
            return this.updateDiagnostics();
          },
          { delay: 0 }
        ),
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
      from: d.start,
      to: d.end,
      severity: d.severity.toLowerCase(),
      message: d.message,
    }));
    this.updatePanel(diagnostics);
    return diagnostics;
  }

  runQuery() {
    if (
      // run query, and put results in query result viewer pane
      this.currentLanguage() === "graphql" &&
      this.showingQueryResultsOrArguments === "results"
    ) {
      let queryResults = this.oxc.run_query(
        this.parserOptions,
        this.viewer.state.doc.toString(),
        getStringFromStorage(STORAGE_KEY_QUERY_ARGUMENTS) ?? '{}' // must be a string of an empty object as this is a string param
      );

      let output =
        typeof queryResults === "string"
          ? queryResults
          : JSON.stringify(queryResults, null, 2);

      let stateUpdate = this.queryResultsViewer.state.update({
        changes: {
          from: 0,
          to: this.queryResultsViewer.state.doc.length,
          insert: output,
        },
      });

      this.queryResultsViewer.dispatch(stateUpdate);
    }
  }

  initViewer() {
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
        linter(
          (data) => {
            try {
              if (this.currentLanguage() === "graphql") {
                setStringToStorage(
                  STORAGE_KEY_QUERY,
                  data.state.doc.toString()
                );
              }
            } finally {
              return [];
            }
          },
          { delay: 0 }
        ),
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

          let ext;

          if (this.currentLanguage() === "graphql") {
            ext = EditorView.editable.of(true);
          } else {
            ext = EditorView.editable.of(false);
          }

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
            case "graphql":
              if (currentLanguage == graphqlLanguage) return null;
              newLanguage = graphql(buildSchema(graphql_schema_text()));
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

  initQueryResultsViewer() {
    return new EditorView({
      extensions: [
        basicSetup,
        githubDark,
        keymap.of([
          ...vscodeKeymap,
          indentWithTab,
          {
            key: "Delete",
            shift: deleteLine,
          },
        ]),
        json(),
        EditorState.transactionExtender.of((tr) => {
          if (!tr.docChanged) return null;

          let ext;

          if (this.showingQueryResultsOrArguments === "arguments") {
            ext = EditorView.editable.of(true);
          } else {
            ext = EditorView.editable.of(false);
          }

          return {
            effects: this.queryResultViewerIsEditableConf.reconfigure(ext),
          };
        }),
        this.queryResultViewerIsEditableConf.of(EditorView.editable.of(false)),
        linter(
          (data) => {
            if (this.showingQueryResultsOrArguments === "arguments") {
              try {
                let parsed = JSON.parse(data.state.doc.toString()); // parse so that if the json is invalid we will not save it because we will have thrown an error instead
                if (parsed) {
                  if (typeof parsed === "object") {
                    if (
                      Object.entries(parsed).some(
                        // todo: this only does depth 1, we should do depth n as in: inside arrays
                        (x) => typeof x[1] === "object" && !Array.isArray(x[1])
                      )
                    ) {
                      return [
                        {
                          from: 0,
                          to: data.state.doc.length,
                          message:
                            "This is invalid for query arguments. The arguments will not be saved until there is are no subobjects in the object.",
                          severity: "error",
                        },
                      ];
                    } else {
                      setStringToStorage(
                        STORAGE_KEY_QUERY_ARGUMENTS,
                        JSON.stringify(
                          JSON.parse(data.state.doc.toString()), // parse so that if the json is invalid we will not save it because we will have thrown an error instead
                          null,
                          2
                        )
                      );
                    }
                  } else {
                    return [
                      {
                        from: 0,
                        to: data.state.doc.length,
                        message:
                          "This is invalid for query arguments. The arguments will not be saved until there is an object at the top level.",
                        severity: "error",
                      },
                    ];
                  }
                } else {
                  return [
                    {
                      from: 0,
                      to: data.state.doc.length,
                      message:
                        "This is invalid for query arguments. The arguments will not be saved until there is an object at the top level.",
                      severity: "error",
                    },
                  ];
                }
              } catch {
                // invalid json in arguments view
                return [
                  {
                    from: 0,
                    to: data.state.doc.length,
                    message:
                      "This is invalid JSON. Will not be saved until it is valid.",
                    severity: "error",
                  },
                ];
              }
            }
            return [];
          },
          { delay: 0 }
        ),
        lintGutter(),
      ],
      parent: document.querySelector("#query-results-viewer"),
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
        return "json";
      case "query":
        return "graphql";
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
    document.getElementById("query-args-or-outputs").style.display = "none";
    document.getElementById("query-results-viewer").style.display = "none";
    // disable #duration and #panel during query view
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
      case "format":
        this.runOptions.format = true;
        this.run();
        text = this.oxc.formattedText;
        break;
      case "query":
        document.getElementById("query-args-or-outputs").style.display =
          "inline";
        document.getElementById("query-results-viewer").style.display =
          "inline";
        document.getElementById("duration").style.display = "none";
        document.getElementById("panel").style.display = "none";
        let savedQuery = getStringFromStorage(STORAGE_KEY_QUERY);
        if (!savedQuery) {
          text = `
query {
  File {
    import {
      from_path @output

      specific_import @fold {
        original_name @output
      }

      default_import @fold {
        local_name @output
      }
    }
  }
}`.trim();
        } else {
          text = savedQuery;
        }
        break;
    }

    this.updateEditorText(this.viewer, text);
    this.runQuery();
  }

  changeBetweenQueryResultsAndQueryArgumentsView() {
    this.showingQueryResultsOrArguments =
      this.showingQueryResultsOrArguments === "results"
        ? "arguments"
        : "results";

    const { classList } = document.getElementById("query-args-or-outputs");
    switch (this.showingQueryResultsOrArguments) {
      case "results":
        this.runQuery();
        classList.add("query-button-red");
        classList.remove("query-button-green");
        break;
      case "arguments":
        this.updateEditorText(
          this.queryResultsViewer,
          getStringFromStorage(STORAGE_KEY_QUERY_ARGUMENTS) ?? "{}"
        );
        classList.add("query-button-green");
        classList.remove("query-button-red");
        break;
      default:
        throw new Error(
          `Unknown value for this.showingQueryResultsOrArguments: ${this.showingQueryResultsOrArguments}`
        );
    }
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
    // if we didn't find a start or an end, return early
    if (start == undefined || end == undefined) return;

    // convert utf8 to utf16 span so they show correctly in the editor
    start = new TextDecoder().decode(this.sourceTextUtf8.slice(0, start)).length;
    end = new TextDecoder().decode(this.sourceTextUtf8.slice(0, end)).length;

    this.highlightEditorRange(
      this.editor,
      EditorSelection.range(start, end)
    );
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

  document.getElementById("ast").onclick = () => {
    playground.updateView("ast");
  };

  document.getElementById("codegen").onclick = () => {
    playground.updateView("codegen");
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


  document.getElementById("query").onclick = () => {
    playground.updateView("query");
  };

  document.getElementById("query-args-or-outputs").onclick = () => {
    playground.changeBetweenQueryResultsAndQueryArgumentsView();
    if (playground.showingQueryResultsOrArguments === "results") {
      document.getElementById("query-args-or-outputs").innerText =
        "Show Query Arguments";
    } else {
      document.getElementById("query-args-or-outputs").innerText =
        "Show Query Results";
    }
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

  document.getElementById("type-check").onchange = function () {
    const checked = document.getElementById("type-check-checkbox").checked;
    playground.runOptions.type_check = checked;
    playground.updateView();
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

import { basicSetup } from "codemirror";
import { EditorView, keymap, showPanel } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { vscodeKeymap } from "@replit/codemirror-vscode-keymap";
import { githubDark } from "@ddietr/codemirror-themes/github-dark";
import { linter, lintGutter } from "@codemirror/lint";

function getConsole(doc) {
  return `Hello`;
}

function consolePanel(view) {
  let dom = document.createElement("div");
  dom.textContent = getConsole(view.state.doc);
  return {
    dom,
    update(update) {
      if (update.docChanged) dom.textContent = getConsole(update.state.doc);
    },
  };
}

let oxcLinter = linter((view) => {
  return [
    {
      from: 0,
      to: 8,
      severity: "warning",
      message: "Haha",
    },
  ];
});

const doc = `
function foo() {
}`.trim();

const editor = new EditorView({
  state: EditorState.create({
    doc,
    extensions: [
      basicSetup,
      keymap.of(vscodeKeymap),
      javascript(),
      githubDark,
      lintGutter(),
      showPanel.of(consolePanel),
      oxcLinter,
    ],
  }),
  parent: document.querySelector("#app"),
});

const right = new EditorView({
  extensions: [
    basicSetup,
    javascript(),
    githubDark,
    EditorView.editable.of(false),
  ],
  parent: document.querySelector("#right"),
});

window.setTimeout(function () {
  editor.focus();
});

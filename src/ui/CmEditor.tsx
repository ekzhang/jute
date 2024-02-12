import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import { python } from "@codemirror/lang-python";
import {
  bracketMatching,
  defaultHighlightStyle,
  foldKeymap,
  indentOnInput,
  indentUnit,
  syntaxHighlighting,
} from "@codemirror/language";
import { lintKeymap } from "@codemirror/lint";
import { EditorState, Prec } from "@codemirror/state";
import {
  EditorView,
  crosshairCursor,
  drawSelection,
  dropCursor,
  highlightSpecialChars,
  keymap,
  rectangularSelection,
} from "@codemirror/view";
import { useContext, useEffect, useRef } from "react";

import { NotebookContext } from "./Notebook";

type CmEditorProps = {
  /** A globally unique identifier for the editor. */
  editorId: string;
};

const editorTheme = EditorView.theme({
  "&": {
    fontSize: "14px",
  },
  "&.cm-focused": {
    outline: "none",
  },
});

export default ({ editorId }: CmEditorProps) => {
  const notebook = useContext(NotebookContext)!;

  const containerEl = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const editor = new EditorView({
      extensions: [
        highlightSpecialChars(),
        history(),
        drawSelection(),
        dropCursor(),
        EditorState.allowMultipleSelections.of(true),
        indentOnInput(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        bracketMatching(),
        closeBrackets(),
        autocompletion(),
        rectangularSelection(),
        crosshairCursor(),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...completionKeymap,
          ...lintKeymap,
          indentWithTab,
        ]),

        Prec.highest(
          keymap.of([
            {
              key: "Shift-Enter",
              run: () => {
                notebook.execute(editorId);
                return true;
              },
            },
            {
              key: "Mod-Enter",
              run: () => {
                notebook.execute(editorId);
                return true;
              },
            },
          ]),
        ),

        python(),
        indentUnit.of("    "),
        EditorState.tabSize.of(4),
        editorTheme,
      ],
      parent: containerEl.current!,
    });

    if (notebook.editors.has(editorId)) {
      console.warn(`Registering duplicate editorId: ${editorId}`);
    }

    console.log("hi");
    notebook.editors.set(editorId, editor);
    return () => {
      notebook.editors.delete(editorId);
    };
  }, [editorId]);

  return <div ref={containerEl} className="cm-editor-container"></div>;
};

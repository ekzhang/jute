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
import { useEffect, useRef } from "react";
import { useStore } from "zustand";

import { useNotebook } from "./Notebook";

type Props = {
  /** A globally unique identifier for the editor. */
  cellId: string;
};

const editorTheme = EditorView.theme({
  "&": {
    fontSize: "14px",
  },
  "&.cm-focused": {
    outline: "none",
  },
  "& .cm-scroller": {
    fontFamily: "Fira Code Variable, ui-monospace, monospace",
    fontVariantLigatures: "none",
  },
});

/**
 * Cell input for a notebook. Note that this component requires CodeMirror, so
 * it's wrapped in lazy loading to improve initial render speed.
 */
export default function CellInput({ cellId }: Props) {
  const notebook = useNotebook();
  const initialText = useStore(
    notebook.store,
    (state) => state.cells[cellId].initialText,
  );

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
                notebook.execute(cellId);
                return true;
              },
            },
            {
              key: "Mod-Enter",
              run: () => {
                notebook.execute(cellId);
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
      doc: initialText,
      parent: containerEl.current!,
    });

    const ref = notebook.refs.get(cellId);
    if (ref) {
      ref.editor = editor;
    } else {
      console.warn(`Ref for cell ${cellId} not found`);
    }

    return () => editor.destroy();
  }, [cellId, notebook]); // eslint-disable-line react-hooks/exhaustive-deps

  return <div ref={containerEl} className="cm-editor-container"></div>;
}

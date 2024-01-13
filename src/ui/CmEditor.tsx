import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { python } from "@codemirror/lang-python";
import {
  bracketMatching,
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
  indentOnInput,
  indentUnit,
  syntaxHighlighting,
} from "@codemirror/language";
import { lintKeymap } from "@codemirror/lint";
import { EditorState } from "@codemirror/state";
import {
  EditorView,
  crosshairCursor,
  drawSelection,
  dropCursor,
  highlightActiveLine,
  highlightActiveLineGutter,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
} from "@codemirror/view";
import { useContext, useEffect, useRef } from "react";

import NotebookContext from "./NotebookContext";

type CmEditorProps = {
  /** A globally unique identifier for the editor. */
  editorId: string;
};

export default ({ editorId }: CmEditorProps) => {
  const notebook = useContext(NotebookContext)!;

  const containerEl = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const editor = new EditorView({
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightSpecialChars(),
        history(),
        foldGutter(),
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
        highlightActiveLine(),
        keymap.of([
          ...closeBracketsKeymap,
          ...defaultKeymap,
          ...historyKeymap,
          ...foldKeymap,
          ...completionKeymap,
          ...lintKeymap,
        ]),

        python(),
        indentUnit.of("    "),
        EditorState.tabSize.of(4),
      ],
      parent: containerEl.current!,
    });

    if (notebook.editor.has(editorId)) {
      console.warn(`Registering duplicate editorId: ${editorId}`);
    }

    console.log("hi");
    notebook.editor.set(editorId, editor);
    return () => {
      notebook.editor.delete(editorId);
    };
  }, [editorId]);

  return <div ref={containerEl} className="cm-editor-container"></div>;
};

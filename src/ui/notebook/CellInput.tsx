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
import { markdown } from "@codemirror/lang-markdown";
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
import { Compartment, EditorState, Prec } from "@codemirror/state";
import {
  EditorView,
  crosshairCursor,
  drawSelection,
  dropCursor,
  highlightSpecialChars,
  keymap,
  lineNumbers,
  rectangularSelection,
} from "@codemirror/view";
import { useEffect, useRef, useState } from "react";
import { useStore } from "zustand";

import { useNotebook } from "@/stores/notebook";

import CellInputFallback from "./CellInputFallback";
import RenderMarkdownCell from "./RenderMarkdownCell";

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
  "& .cm-content": {
    paddingBlock: "16px",
    flexShrink: "0",
  },
  "& .cm-line": {
    paddingLeft: "0",
    paddingRight: "16px",
  },
  "& .cm-gutters": {
    borderRight: "none",
    background: "white",
    color: "#b1b1b1",
    cursor: "default",
  },
  "& .cm-lineNumbers .cm-gutterElement": {
    minWidth: "57px",
    paddingRight: "18px",
  },
});

const language = new Compartment();
const lineNumbersDynamic = new Compartment();

/**
 * Cell input for a notebook. Note that this component requires CodeMirror, so
 * it's wrapped in lazy loading to improve initial render speed.
 */
export default function CellInput({ cellId }: Props) {
  const notebook = useNotebook();

  const [view, setView] = useState<EditorView | null>(null);

  const type = useStore(notebook.store, (state) => state.cells[cellId].type);
  const initialText = useStore(
    notebook.store,
    (state) => state.cells[cellId].initialText,
  );

  // Only updated when the editor view is switched to Markdown rendering mode.
  const [renderedMarkdown, setRenderedMarkdown] = useState(initialText);
  const [editingMarkdown, setEditingMarkdown] = useState(false);

  const containerEl = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onRun = () => {
      if (type === "code") {
        notebook.execute(cellId);
      } else if (type === "markdown") {
        if (view) {
          console.log(view.state.doc.toString());
          setRenderedMarkdown(view.state.doc.toString());
        }
        setEditingMarkdown(false);
      }
      return true;
    };

    const view = new EditorView({
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
            { key: "Shift-Enter", run: onRun },
            { key: "Mod-Enter", run: onRun },
          ]),
        ),

        // TODO: Figure out state dataflow for cumulative line numbers.
        lineNumbersDynamic.of(
          type === "code"
            ? lineNumbers({ formatNumber: (x) => String(x + 0) })
            : lineNumbers({ formatNumber: () => "" }),
        ),
        language.of(type === "code" ? python() : markdown()),
        indentUnit.of("    "),
        EditorState.tabSize.of(4),
        editorTheme,
      ],
      doc: initialText,
      parent: containerEl.current!,
    });

    const ref = notebook.refs.get(cellId);
    if (ref) {
      ref.editor = view;
    } else {
      console.warn(`Ref for cell ${cellId} not found`);
    }

    setView(view);
    return () => view.destroy();
  }, [cellId, notebook]); // eslint-disable-line react-hooks/exhaustive-deps

  // If the language changes, reconfigure the compartment.
  useEffect(() => {
    if (view) {
      view.dispatch({
        effects: [
          language.reconfigure(type === "code" ? python() : markdown()),
          lineNumbersDynamic.reconfigure(
            type === "code"
              ? lineNumbers({ formatNumber: (x) => String(x + 0) })
              : lineNumbers({ formatNumber: () => "" }),
          ),
        ],
      });
    }
  }, [view, type]);

  const isRenderingMarkdown = type === "markdown" && !editingMarkdown;

  return (
    <>
      <div
        ref={containerEl}
        style={{ display: isRenderingMarkdown ? "none" : "block" }}
      >
        {/* Eliminate flickering when the editor first loads in. */}
        <div className="hidden only:block">
          <CellInputFallback cellId={cellId} />
        </div>
      </div>
      {isRenderingMarkdown && (
        <RenderMarkdownCell
          source={renderedMarkdown}
          onStartEdit={() => setEditingMarkdown(true)}
        />
      )}
    </>
  );
}

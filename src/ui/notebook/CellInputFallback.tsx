import { useStore } from "zustand";

import { useNotebook } from "@/stores/notebook";

import RenderMarkdownCell from "./RenderMarkdownCell";

type Props = {
  cellId: string;
};

/** Fallback component with no editing or syntax highlighting, before CodeMirror loads. */
export default function CellInput({ cellId }: Props) {
  const notebook = useNotebook();
  const type = useStore(notebook.store, (state) => state.cells[cellId].type);
  const initialText = useStore(
    notebook.store,
    (state) => state.cells[cellId].initialText,
  );

  // 57px padding left matches the gutter size of the full editor.
  return type === "markdown" ? (
    <RenderMarkdownCell source={initialText} />
  ) : (
    <pre
      className="max-w-full overflow-hidden py-4 pl-[57px] pr-0.5 text-sm leading-[1.2rem]"
      style={{
        fontFamily: "Fira Code Variable, ui-monospace, monospace",
        fontVariantLigatures: "none",
      }}
    >
      {initialText}
    </pre>
  );
}

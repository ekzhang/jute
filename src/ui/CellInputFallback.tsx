import { useStore } from "zustand";

import { useNotebook } from "./Notebook";

type Props = {
  cellId: string;
};

/** Fallback component with no editing or syntax highlighting, before CodeMirror loads. */
export default function CellInput({ cellId }: Props) {
  const notebook = useNotebook();
  const initialText = useStore(
    notebook.store,
    (state) => state.cells[cellId].initialText,
  );

  return (
    <pre
      className="py-1 pl-1.5 pr-0.5 text-sm leading-[1.2rem]"
      style={{
        fontFamily: "Fira Code Variable, ui-monospace, monospace",
        fontVariantLigatures: "none",
      }}
    >
      {initialText}
    </pre>
  );
}

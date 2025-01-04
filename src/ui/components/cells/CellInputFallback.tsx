import { useStore } from "zustand";
import { useNotebook } from "~/hooks/notebook";

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

  // 57px padding left matches the gutter size of the full editor.
  return (
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

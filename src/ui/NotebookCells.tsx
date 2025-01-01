import { CircleIcon, CornerDownRightIcon, PlusIcon } from "lucide-react";
import { Suspense, lazy } from "react";
import { useStore } from "zustand";

import CellInputFallback from "./CellInputFallback";
import { useNotebook } from "./Notebook";
import OutputView from "./OutputView";

const CellInput = lazy(() => import("./CellInput"));

export default function NotebookCells() {
  const notebook = useNotebook();
  const cellIds = useStore(notebook.store, (state) => state.cellIds);
  const cells = useStore(notebook.store, (state) => state.cells);

  return (
    <div className="py-16">
      {cellIds.map((id) => (
        <div key={id}>
          <div className="flex items-start gap-2 px-4">
            <CircleIcon className="my-[6px] h-3.5 w-3.5 fill-zinc-200 stroke-none" />
            <div className="flex-1">
              <Suspense fallback={<CellInputFallback cellId={id} />}>
                <CellInput cellId={id} />
              </Suspense>
            </div>
          </div>
          {cells[id]?.output && (
            <div className="flex gap-3 px-4 pt-4">
              <div>
                <CornerDownRightIcon className="ml-1 h-5 w-5 text-green-700" />
              </div>
              <div className="pt-0.5">
                <OutputView value={cells[id].output} />
              </div>
            </div>
          )}
          <hr className="mx-2 my-4 border-zinc-200" />
        </div>
      ))}

      <div className="mx-2 my-4">
        <button
          className="flex w-full items-center justify-center gap-1.5 rounded border border-zinc-200 p-2 transition-colors hover:border-zinc-300 hover:bg-zinc-50"
          onClick={() => {
            notebook.addCell("");
          }}
        >
          <PlusIcon size={18} />
          <span>New cell</span>
        </button>
      </div>
    </div>
  );
}

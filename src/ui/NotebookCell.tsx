import { Check, CircleIcon, CornerDownRightIcon, X } from "lucide-react";
import { Suspense, lazy } from "react";
import { useStore } from "zustand";

import CellInputFallback from "./CellInputFallback";
import { useNotebook } from "./Notebook";
import OutputView from "./OutputView";

const CellInput = lazy(() => import("./CellInput"));

type Props = {
  id: string;
};

export default function NotebookCell({ id }: Props) {
  const notebook = useNotebook();
  const cell = useStore(notebook.store, (state) => state.cells[id]);

  if (!cell) {
    return null;
  }

  return (
    <div className="flex">
      <div className="flex-1">
        <div className="flex items-start gap-2 px-4">
          <CircleIcon className="mt-1.5 size-3.5 fill-gray-200 stroke-none" />
          <div className="flex-1">
            <Suspense fallback={<CellInputFallback cellId={id} />}>
              <CellInput cellId={id} />
            </Suspense>
          </div>
        </div>

        {cell?.output && (
          <div className="flex gap-3 px-4 pt-4">
            <div>
              <CornerDownRightIcon className="ml-1 size-5 text-gray-500" />
            </div>
            <div className="pt-0.5">
              <OutputView value={cell.output} />
            </div>
          </div>
        )}

        <hr className="mx-2 my-4 border-gray-200" />
      </div>

      <div className="w-[200px] border-l border-gray-200 bg-gray-100 px-4 text-sm">
        {cell.output && (
          <div className="flex items-center gap-1">
            {cell.output.status === "success" ? (
              <Check className="size-4 text-green-700" />
            ) : (
              <X className="size-4 text-red-700" />
            )}

            <div className="font-extralight text-gray-500">
              {Math.round(cell.output.durationMs || 0)} ms
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

import { Check, CircleIcon, CornerDownRightIcon, X } from "lucide-react";
import { Suspense, lazy } from "react";
import { useStore } from "zustand";

import CellInputFallback from "./CellInputFallback";
import { useNotebook } from "./Notebook";
import OutputView from "./OutputView";

const CellInput = lazy(() => import("./CellInput"));

type Props = {
  id: string;
}

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
          <CircleIcon className="mt-1.5 size-3.5 fill-zinc-200 stroke-none" />
          <div className="flex-1">
            <Suspense fallback={<CellInputFallback cellId={id} />}>
              <CellInput cellId={id} />
            </Suspense>
          </div>
        </div>

        {cell?.output && (
          <div className="flex gap-3 px-4 pt-4">
            <div>
              <CornerDownRightIcon className="ml-1 size-5 text-slate-500" />
            </div>
            <div className="pt-0.5">
              <OutputView value={cell.output} />
            </div>
          </div>
        )}

        <hr className="mx-2 my-4 border-zinc-200" />
      </div>

      <div className="border-l border-zinc-200 bg-zinc-100 w-[200px] px-4 text-sm">
        {cell.output && (
          <div className="flex gap-1 items-center">
            {cell.output.status === 'success' ? (
              <Check className="size-4 text-green-700" />
            ) : (
              <X className="size-4 text-red-700" />
            )}

            <div className="text-slate-500 font-extralight">{Math.round(cell.output.durationMs || 0)} ms</div>
          </div>
        )}
      </div>
    </div>
  );
}


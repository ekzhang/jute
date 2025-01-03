import { PlusIcon } from "lucide-react";
import { useStore } from "zustand";

import { useNotebook } from "./Notebook";
import NotebookCell from "./NotebookCell";

export default function NotebookCells() {
  const notebook = useNotebook();
  const cellIds = useStore(notebook.store, (state) => state.cellIds);

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <div className="flex">
        <div className="h-16 flex-1" />
        <div className="w-[200px] border-l border-gray-200 bg-gray-100" />
      </div>

      {cellIds.map((id) => (
        <NotebookCell key={id} id={id} />
      ))}

      <div className="flex flex-1">
        <div className="mx-2 my-4 flex-1">
          <button
            className="flex w-full items-center justify-center gap-1.5 rounded border border-gray-200 p-2 transition-colors hover:border-zinc-300 hover:bg-zinc-50"
            onClick={() => {
              notebook.addCell("");
            }}
          >
            <PlusIcon size={18} />
            <span>New cell</span>
          </button>
        </div>

        <div className="py-auto w-[200px] border-l border-gray-200 bg-gray-100" />
      </div>
    </div>
  );
}

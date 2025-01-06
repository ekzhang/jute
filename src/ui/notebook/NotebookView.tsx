import { useNotebook } from "@/stores/notebook";

import NotebookCells from "./NotebookCells";
import NotebookLocation from "./NotebookLocation";
import { useStore } from "zustand";
import { Error } from "@/ui/shared/Error";

export default function NotebookView() {
  const notebook = useNotebook();

  const error = useStore(notebook.store, (state) => state.error);

  return (
    <div className="grid h-full grid-cols-[1fr,200px] overflow-y-auto">
      <div className="min-w-0 py-16">
        <NotebookLocation
          directory={notebook.directory}
          filename={notebook.filename}
        />

        {error ? (
          <Error error={error} />
        ) : (
          <NotebookCells />
        )}
      </div>
      <div
        className="border-l border-gray-200 bg-gray-100"
        data-tauri-drag-region
      />
    </div>
  );
}

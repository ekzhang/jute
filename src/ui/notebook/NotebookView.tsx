import { useNotebook } from "@/stores/notebook";

import NotebookCells from "./NotebookCells";
import NotebookLocation from "./NotebookLocation";

export default function NotebookView() {
  const notebook = useNotebook();

  return (
    <div className="grid h-full grid-cols-[1fr,200px] overflow-y-auto">
      <div className="min-w-0 py-16">
        <NotebookLocation
          directory={notebook.directory}
          filename={notebook.filename}
        />
        <NotebookCells />
      </div>
      <div
        className="border-l border-gray-200 bg-gray-100"
        data-tauri-drag-region
      />
    </div>
  );
}

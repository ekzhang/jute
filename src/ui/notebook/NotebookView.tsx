import { useStore } from "zustand";
import { useShallow } from "zustand/react/shallow";

import { useNotebook } from "@/stores/notebook";
import { UnhandledError } from "@/ui/shared/UnhandledError";

import NotebookCells from "./NotebookCells";
import NotebookLocation from "./NotebookLocation";

export default function NotebookView() {
  const notebook = useNotebook();

  const [path, loadError] = useStore(
    notebook.store,
    useShallow((state) => [state.path, state.loadError]),
  );

  // should be set to default home directory, and kernel should start there too
  let directory = "/fill/in_this_later";
  let filename: string | null = null;
  if (path) {
    const idx = path.lastIndexOf("/");
    directory = path.slice(0, idx);
    filename = path.slice(idx + 1);
  }

  return (
    <div className="grid h-full grid-cols-[1fr,200px] overflow-y-auto">
      <div className="min-w-0 py-16">
        <NotebookLocation directory={directory} filename={filename} />

        {/* TODO: Handle these errors gracefully. */}
        {loadError ? <UnhandledError error={loadError} /> : <NotebookCells />}
      </div>
      <div
        className="border-l border-gray-200 bg-gray-100"
        data-tauri-drag-region
      />
    </div>
  );
}

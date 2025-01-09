import { useEffect, useMemo } from "react";
import { useSearch } from "wouter";

import { Notebook, NotebookContext } from "@/stores/notebook";
import NotebookFooter from "@/ui/notebook/NotebookFooter";
import NotebookHeader from "@/ui/notebook/NotebookHeader";
import NotebookView from "@/ui/notebook/NotebookView";

export default function NotebookPage() {
  const { path, inline } = Object.fromEntries(new URLSearchParams(useSearch()));

  // Singleton notebook object used for the lifetime of this component.
  const notebook = useMemo(() => new Notebook(), []);

  useEffect(() => {
    if (path) {
      notebook.loadNotebookFromPath(path);
    } else if (inline) {
      notebook.loadNotebook(JSON.parse(inline));
    }
  }, [notebook, path, inline]);

  return (
    <main className="absolute inset-0 bg-white">
      <NotebookContext.Provider value={notebook}>
        <NotebookHeader kernelName="Local Kernel (Python 3.11.7)" />
        <NotebookView />
        <NotebookFooter />
      </NotebookContext.Provider>
    </main>
  );
}

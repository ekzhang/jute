import { useMemo } from "react";
import { useSearch } from "wouter";

import { Notebook, NotebookContext } from "@/stores/notebook";
import NotebookFooter from "@/ui/notebook/NotebookFooter";
import NotebookHeader from "@/ui/notebook/NotebookHeader";
import NotebookView from "@/ui/notebook/NotebookView";

export default function NotebookPage() {
  const { path } = Object.fromEntries(new URLSearchParams(useSearch()));
  // Single mutable object that is shared between all parts of the notebook.
  const notebook = useMemo(() => new Notebook(path), [path]);

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

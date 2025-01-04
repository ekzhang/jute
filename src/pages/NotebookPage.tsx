import { useSearch } from "wouter";

import NotebookFooter from "@/ui/notebook/NotebookFooter";
import NotebookHeader from "@/ui/notebook/NotebookHeader";
import NotebookView from "@/ui/notebook/NotebookView";

export default function NotebookPage() {
  const { path } = Object.fromEntries(new URLSearchParams(useSearch()));

  return (
    <main className="absolute inset-0 bg-white">
      <NotebookHeader kernelName="Local Kernel (Python 3.11.7)" />
      <NotebookView path={path} />
      <NotebookFooter />
    </main>
  );
}

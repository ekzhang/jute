import { useEffect, useMemo } from "react";
import { useSearch } from "wouter";

import { Notebook, NotebookContext } from "@/stores/notebook";
import NotebookFooter from "@/ui/notebook/NotebookFooter";
import NotebookHeader from "@/ui/notebook/NotebookHeader";
import NotebookView from "@/ui/notebook/NotebookView";

export default function NotebookPage() {
  const { path } = Object.fromEntries(new URLSearchParams(useSearch()));
  // Single mutable object that is shared between all parts of the notebook.
  const notebook = useMemo(() => new Notebook(path), [path]);

  useEffect(() => {
    notebook.addCell(`print("Hello, world!")`);
    notebook.addCell(`for i in range(100):
    if i % 15 == 0:
        print("FizzBuzz")
    elif i % 3 == 0:
        print("Fizz")
    elif i % 5 == 0:
        print("Buzz")
    else:
        print(i)`);
    notebook.addCell(`import matplotlib.pyplot as plt
import numpy as np

plt.plot(np.random.normal(size=(400,)).cumsum())`);
  }, [notebook]);

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

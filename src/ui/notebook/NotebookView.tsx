import { useEffect, useMemo } from "react";

import { Notebook, NotebookContext } from "@/stores/notebook";

import NotebookCells from "./NotebookCells";
import NotebookLocation from "./NotebookLocation";

type Props = {
  path: string;
};

export default function NotebookView({ path }: Props) {
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
    <NotebookContext.Provider value={notebook}>
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
    </NotebookContext.Provider>
  );
}

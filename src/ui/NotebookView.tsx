import { useEffect, useMemo } from "react";

import { Notebook, NotebookContext } from "./Notebook";
import NotebookCells from "./NotebookCells";

export default function NotebookView() {
  // Single mutable object that is shared between all parts of the notebook.
  const notebook = useMemo(() => new Notebook(), []);

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

plt.plot(np.random.randn(200))`);
  }, [notebook]);

  return (
    <NotebookContext.Provider value={notebook}>
      <NotebookCells />
    </NotebookContext.Provider>
  );
}

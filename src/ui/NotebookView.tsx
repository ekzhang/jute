import { useEffect, useMemo, useReducer } from "react";

import { Notebook, NotebookContext } from "./Notebook";
import NotebookCells from "./NotebookCells";

export default () => {
  const [, rerender] = useReducer((x) => x + 1, 0);

  // Single mutable object that is shared between all parts of the notebook.
  const notebook = useMemo(() => new Notebook(rerender), []);

  useEffect(() => {
    notebook.editors.get("1")?.dispatch({
      changes: { from: 0, to: 0, insert: `print("Hello, world!")` },
    });
    notebook.editors.get("2")?.dispatch({
      changes: {
        from: 0,
        to: 0,
        insert: `for i in range(100):
    if i % 15 == 0:
        print("FizzBuzz")
    elif i % 3 == 0:
        print("Fizz")
    elif i % 5 == 0:
        print("Buzz")
    else:
        print(i)`,
      },
    });
  }, []);

  return (
    <NotebookContext.Provider value={notebook}>
      <div className="grid h-full grid-cols-[1fr,200px] overflow-y-auto">
        <NotebookCells />
        <div
          className="border-l border-zinc-200 bg-zinc-100"
          data-tauri-drag-region
        />
      </div>
    </NotebookContext.Provider>
  );
};

import { useEffect, useMemo, useReducer } from "react";

import CmEditor from "./CmEditor";
import { Notebook, NotebookContext } from "./Notebook";
import OutputView from "./OutputView";

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
      <div className="space-y-6">
        <div className="rounded-md border border-zinc-200 p-4">
          <CmEditor editorId="1" />
          {notebook.outputs.get("1") && (
            <>
              <div className="my-4 border-t border-zinc-200"></div>
              <OutputView value={notebook.outputs.get("1")} />
            </>
          )}
        </div>

        <div className="rounded-md border border-zinc-200 p-4">
          <CmEditor editorId="2" />
          {notebook.outputs.get("2") && (
            <>
              <div className="my-4 border-t border-zinc-200"></div>
              <OutputView value={notebook.outputs.get("2")} />
            </>
          )}
        </div>
      </div>
    </NotebookContext.Provider>
  );
};

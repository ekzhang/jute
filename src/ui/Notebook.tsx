import { EditorView } from "@codemirror/view";
import { useMemo } from "react";

import CmEditor from "./CmEditor";
import NotebookContext from "./NotebookContext";

function initialNotebookContext() {
  return {
    editor: new Map<string, EditorView>(),
  };
}

export default () => {
  // Single mutable object that is shared between all parts of the notebook.
  const notebook = useMemo(initialNotebookContext, []);

  return (
    <NotebookContext.Provider value={notebook}>
      <CmEditor editorId="1" />
      <CmEditor editorId="2" />

      <button
        onClick={() => {
          console.log(notebook.editor.get("1")?.state.doc.toString());
        }}
      >
        hello
      </button>
    </NotebookContext.Provider>
  );
};

import { EditorView } from "@codemirror/view";
import { createContext } from "react";

type NotebookContextValue = {
  editor: Map<string, EditorView>;
};

const NotebookContext = createContext<NotebookContextValue | undefined>(
  undefined,
);

export default NotebookContext;

import { EditorView } from "@codemirror/view";
import { Channel, invoke } from "@tauri-apps/api/core";
import { createContext, useContext } from "react";
import { StoreApi, createStore } from "zustand";
import { immer } from "zustand/middleware/immer";

type RunPythonEvent =
  | { event: "stdout"; data: string }
  | { event: "stderr"; data: string }
  | { event: "done"; data: { status: number } };

type NotebookStore = NotebookStoreState & NotebookStoreActions;

export type NotebookStoreState = {
  cellIds: string[];
  cells: {
    [cellId: string]: {
      initialText: string;
      output?: NotebookOutput;
    };
  };
};

export type NotebookOutput = { status: "success" | "error"; data: string };

/** Actions are kept private, only to be used from the `Notebook` class. */
type NotebookStoreActions = {
  addCell: (id: string, initialText: string) => void;
  setOutput: (cellId: string, output: NotebookOutput) => void;
};

type CellHandle = {
  editor?: EditorView;
};

export class Notebook {
  /** Zustand object used to reactively update DOM nodes. */
  store: StoreApi<NotebookStore>;

  /** Direct handles to editors and other HTML elements after render. */
  refs: Map<string, CellHandle>;

  constructor() {
    this.store = createStore<NotebookStore>()(
      immer<NotebookStore>((set) => ({
        cellIds: [],
        cells: {},

        addCell: (cellId, initialText) =>
          set((state) => {
            state.cellIds.push(cellId);
            state.cells[cellId] = {
              initialText,
            };
          }),

        setOutput: (cellId, output) =>
          set((state) => {
            state.cells[cellId].output = output;
          }),
      })),
    );
    this.refs = new Map();
  }

  get state() {
    // Helper function, used internally to get the current notebook store state.
    return this.store.getState();
  }

  addCell(initialText: string): string {
    const cellId = Math.random().toString(36).slice(2);
    this.refs.set(cellId, {});
    this.store.getState().addCell(cellId, initialText);
    return cellId;
  }

  async execute(cellId: string) {
    const editor = this.refs.get(cellId)?.editor;
    if (!editor) {
      throw new Error(`Cell ${cellId} not found`);
    }
    const code = editor.state.doc.toString();
    try {
      const onEvent = new Channel<RunPythonEvent>();
      let output = "";
      this.state.setOutput(cellId, { status: "success", data: output });
      onEvent.onmessage = (message) => {
        if (message.event === "stdout" || message.event === "stderr") {
          output += message.data;
          this.state.setOutput(cellId, { status: "success", data: output });
        }
      };

      await invoke("run_python", { sourceCode: code, onEvent });
      this.state.setOutput(cellId, { status: "success", data: output });
    } catch (error: any) {
      this.state.setOutput(cellId, { status: "error", data: error });
    }
  }
}

export const NotebookContext = createContext<Notebook | undefined>(undefined);

export function useNotebook(): Notebook {
  const notebook = useContext(NotebookContext);
  if (!notebook) {
    throw new Error("useNotebook must be used within a NotebookContext");
  }
  return notebook;
}

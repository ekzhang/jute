import { EditorView } from "@codemirror/view";
import { Channel, invoke } from "@tauri-apps/api/core";
import { createContext, useContext } from "react";
import { StoreApi, createStore } from "zustand";
import { immer } from "zustand/middleware/immer";

type RunCellEvent =
  | { event: "stdout"; data: string }
  | { event: "stderr"; data: string }
  | {
      event: "execute_result";
      data: {
        execution_count: number;
        data: Record<string, any>;
        metadata: Record<string, any>;
      };
    }
  | {
      event: "error";
      data: {
        ename: string;
        evalue: string;
        traceback: string[];
      };
    }
  | { event: "disconnect"; data: string };

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
  /** ID of the running kernel, populated after the kernel is started. */
  kernelId: string;

  /** Promise that resolves when the kernel is started. */
  kernelStartPromise: Promise<void>;

  /** Zustand object used to reactively update DOM nodes. */
  store: StoreApi<NotebookStore>;

  /** Direct handles to editors and other HTML elements after render. */
  refs: Map<string, CellHandle>;

  constructor() {
    this.kernelId = "";
    this.kernelStartPromise = (async () => {
      this.kernelId = await invoke("start_kernel", { specName: "python3" });
    })();

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
    if (!this.kernelId) {
      await this.kernelStartPromise;
    }

    const editor = this.refs.get(cellId)?.editor;
    if (!editor) {
      throw new Error(`Cell ${cellId} not found`);
    }
    const code = editor.state.doc.toString();
    try {
      const onEvent = new Channel<RunCellEvent>();
      let output = "";
      this.state.setOutput(cellId, { status: "success", data: output });
      onEvent.onmessage = (message: RunCellEvent) => {
        if (message.event === "stdout" || message.event === "stderr") {
          output += message.data;
          this.state.setOutput(cellId, { status: "success", data: output });
        } else if (message.event === "execute_result") {
          // This means that there was a return value for the cell.
          output += message.data.data["text/plain"];
        }
      };

      await invoke("run_cell", { kernelId: this.kernelId, code, onEvent });
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

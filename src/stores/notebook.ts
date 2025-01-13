import type { EditorView } from "@codemirror/view";
import { Channel, invoke } from "@tauri-apps/api/core";
import { WritableDraft } from "immer";
import { createContext, useContext } from "react";
import { v4 as uuidv4 } from "uuid";
import { StoreApi, createStore } from "zustand";
import { immer } from "zustand/middleware/immer";

import type {
  NotebookRoot,
  Output,
  OutputDisplayData,
  RunCellEvent,
} from "@/bindings";

type NotebookStore = NotebookStoreState & NotebookStoreActions;

/** Actions are kept private, only to be used from the `Notebook` class. */
type NotebookStoreActions = ReturnType<typeof notebookStoreActions>;

/** Zustand reactive data used by the UI to render notebooks. */
export type NotebookStoreState = {
  /** A list of cell IDs in order. */
  cellIds: string[];

  /** Information about each cell, keyed by ID. */
  cells: {
    [cellId: string]: {
      type: CellType;
      initialText: string;
      result?: CellResult;
    };
  };

  /** True when loading the notebook from disk. */
  isLoading: boolean;

  /** Error related to loading the notebook. */
  loadError?: string;

  /** Path to the notebook file, if saved to a file path. */
  path?: string;

  /** ID of the running kernel, populated after the kernel is started. */
  kernelId?: string;
};

export type CellType = "code" | "markdown";

export type CellResult = {
  status: "running" | "success" | "error";
  timings: {
    startedAt: number;
    finishedAt?: number;
  };
  executionCount?: number;
  outputs?: Output[];
  displays?: Record<string, number>;
};

function notebookStoreActions(
  // Updater used by Zustand / Immer to mutate the state.
  set: (updater: (state: WritableDraft<NotebookStoreState>) => void) => void,
) {
  return {
    /** Add a new cell to the notebook. */
    addCell: (cellId: string, type: CellType, initialText: string) =>
      set((state) => {
        state.cellIds.push(cellId);
        state.cells[cellId] = {
          type,
          initialText,
        };
      }),

    /** Set the type of a cell. */
    setCellType: (cellId: string, type: CellType) =>
      set((state) => {
        state.cells[cellId].type = type;
      }),

    /** Clear the result of a cell. */
    clearResult: (cellId: string) =>
      set((state) => {
        state.cells[cellId].result = undefined;
      }),

    /** Update properties of a cell result, except the actual `outputs` array. */
    updateResult: (cellId: string, result: CellResult) =>
      set((state) => {
        const obj = state.cells[cellId].result;
        if (obj) {
          for (const [key, value] of Object.entries(result)) {
            // @ts-ignore
            obj[key] = value;
          }
        } else {
          state.cells[cellId].result = result;
        }
      }),

    /** Append outputs to a cell. */
    appendOutput: (cellId: string, output: Output, displayId?: string) =>
      set((state) => {
        const obj = state.cells[cellId].result;
        if (obj) {
          if (displayId) {
            if (output.output_type !== "display_data") {
              throw new Error("displayId can only be used with display_data");
            }
            obj.displays ??= {};
            obj.displays[displayId] = obj.outputs?.length ?? 0;
          }

          obj.outputs = obj.outputs ?? [];
          if (obj.outputs.length > 0) {
            const lastOutput = obj.outputs[obj.outputs.length - 1];
            if (
              lastOutput.output_type === "stream" &&
              output.output_type === "stream" &&
              lastOutput.name === output.name
            ) {
              // Concatenate to the last stream output if on the same stream.
              lastOutput.text = [
                ...(typeof lastOutput.text === "string"
                  ? [lastOutput.text]
                  : lastOutput.text),
                ...(typeof output.text === "string"
                  ? [output.text]
                  : output.text),
              ];
              return;
            }
          }

          obj.outputs.push(output);
        }
      }),

    /** Clear the output of a cell. */
    clearOutput: (cellId: string) =>
      set((state) => {
        const obj = state.cells[cellId].result;
        if (obj) {
          obj.outputs = [];
          obj.displays = {};
        }
      }),

    /** Update an existing `display_data` output. */
    updateOutputDisplay: (
      cellId: string,
      displayId: string,
      displayData: OutputDisplayData,
    ) =>
      set((state) => {
        const obj = state.cells[cellId].result;
        if (obj) {
          const index = obj.displays?.[displayId];
          if (index !== undefined) {
            const output = obj.outputs?.[index];
            if (output && output.output_type === "display_data") {
              output.data = displayData.data;
              output.metadata = displayData.metadata;
            }
          }
        }
      }),

    /**
     * Start loading the notebook from an external source.
     *
     * After this function is called, no new cell executions should happen until
     * the notebook finishes loading and one of the functions below is called. If
     * successful, this clears the current cells.
     */
    startLoading: () =>
      set((state) => {
        // TODO: Fix this to handle errors better.
        if (state.isLoading) throw new Error("Notebook is already loading");
        state.isLoading = true;
      }),

    /** Load the notebook from a JSON object. */
    loadNotebook: (notebook: NotebookRoot) =>
      set((state) => {
        // Filter out 'raw' cells, as they aren't supported yet.
        const cells = notebook.cells.filter(
          (cell) => cell.cell_type === "code" || cell.cell_type === "markdown",
        );

        // Some older notebooks have no cell IDs, so we generate them on import.
        const cellIds = cells.map((cell) => cell.id ?? uuidv4());

        state.cellIds = cellIds;
        state.cells = Object.fromEntries(
          cells.map((cell, i) => [
            cellIds[i],
            { type: cell.cell_type, initialText: multiline(cell.source) },
          ]),
        );
        state.isLoading = false;
        state.loadError = undefined;
      }),

    /** Set the error on failure to load a notebook. */
    setLoadError: (error: string) =>
      set((state) => {
        state.loadError = error;
        state.isLoading = false;
      }),

    /** Set the path of the notebook, when it is opened or saved. */
    setPath: (path: string) =>
      set((state) => {
        state.path = path;
      }),
  };
}

/** Initialize the Zustand store for a notebook and define mutators. */
function createNotebookStore(): StoreApi<NotebookStore> {
  // @ts-ignore TypeScript says that the instantiation is too deep, infinite?
  return createStore<NotebookStore>()(
    immer<NotebookStore>((set) => {
      const initialState: NotebookStoreState = {
        cellIds: [],
        cells: {},
        isLoading: false,
      };
      const actions: NotebookStoreActions = notebookStoreActions(set);
      return { ...initialState, ...actions };
    }),
  );
}

type CellHandle = {
  editor?: EditorView;
};

/**
 * Centralized stateful object representing a notebook.
 *
 * The Notebook class is responsible for communicating with a running Jupyter
 * kernel and handling edits to notebooks. It also manages the Zustand state
 * for rendering a notebook in the UI.
 *
 * Generally, all user actions will go through methods on this class, which may
 * dispatch to Zustand. The UI subscribes to Zustand for updates.
 */
export class Notebook {
  /** Promise that resolves when the kernel is started. */
  kernelStartPromise: Promise<void>;

  /** Zustand object used to reactively update DOM nodes. */
  store: StoreApi<NotebookStore>;

  /** Direct handles to editors and other HTML elements after render. */
  refs: Map<string, CellHandle>;

  constructor() {
    const store = createNotebookStore();
    this.store = store;
    this.refs = new Map();

    this.kernelStartPromise = (async () => {
      const kernelId = await invoke<string>("start_kernel", {
        specName: "python3",
      });
      store.setState({ kernelId });
    })();
  }

  /** Access the current value of the notebook store, non-reactively. */
  get state() {
    return this.store.getState();
  }

  /** Load a notebook from a direct object. */
  loadNotebook(notebook: NotebookRoot) {
    this.state.loadNotebook(notebook);
    this.refs = new Map(this.state.cellIds.map((cellId) => [cellId, {}]));
  }

  /** Load a notebook from a file path. */
  async loadNotebookFromPath(path: string) {
    try {
      this.state.startLoading();
    } catch {
      return;
    }
    try {
      const notebook = await invoke<NotebookRoot>("get_notebook", { path });
      this.loadNotebook(notebook);
      this.state.setPath(path);
    } catch (e: any) {
      this.state.setLoadError(e.toString());
    }
  }

  addCell(type: CellType, initialText: string): string {
    const cellId = Math.random().toString(36).slice(2);
    this.refs.set(cellId, {});
    this.state.addCell(cellId, type, initialText);
    return cellId;
  }

  setCellType(cellId: string, type: CellType) {
    this.state.setCellType(cellId, type);
  }

  clearResult(cellId: string) {
    this.state.clearResult(cellId);
  }

  async execute(cellId: string) {
    if (!this.state.kernelId) {
      await this.kernelStartPromise;
    }

    const editor = this.refs.get(cellId)?.editor;
    if (!editor) {
      throw new Error(`Cell ${cellId} not found`);
    }
    const code = editor.state.doc.toString();

    let status: CellResult["status"] = "running";
    let timings: CellResult["timings"] = { startedAt: Date.now() };
    let executionCount: CellResult["executionCount"] = undefined;

    const update = () =>
      this.state.updateResult(cellId, {
        status,
        timings,
        executionCount,
      });
    update();
    this.state.clearOutput(cellId);

    let willClearOutput = false;

    try {
      const onEvent = new Channel<RunCellEvent>();

      onEvent.onmessage = (message: RunCellEvent) => {
        console.log(message);
        if (willClearOutput) {
          this.state.clearOutput(cellId);
          willClearOutput = false;
        }

        if (message.event === "stdout" || message.event === "stderr") {
          this.state.appendOutput(cellId, {
            output_type: "stream",
            name: message.event,
            text: message.data,
          });
          console.log(this.state.cells[cellId].result);
        } else if (message.event === "error") {
          status = "error";
          update();
          this.state.appendOutput(cellId, {
            output_type: "error",
            ename: message.data.ename,
            evalue: message.data.evalue,
            traceback: message.data.traceback,
          });
        } else if (message.event === "execute_result") {
          // This means that there was a return value for the cell.
          executionCount = message.data.execution_count;
          update();
          this.state.appendOutput(cellId, {
            output_type: "execute_result",
            execution_count: message.data.execution_count,
            data: message.data.data,
            metadata: message.data.metadata,
          });
        } else if (message.event === "display_data") {
          const displayId = message.data.transient?.display_id || uuidv4();
          this.state.appendOutput(
            cellId,
            {
              output_type: "display_data",
              data: message.data.data,
              metadata: message.data.metadata,
            },
            displayId,
          );
        } else if (message.event === "update_display_data") {
          const displayId = message.data.transient?.display_id;
          if (displayId) {
            this.state.updateOutputDisplay(cellId, displayId, {
              data: message.data.data,
              metadata: message.data.metadata,
            });
          }
        } else if (message.event === "clear_output") {
          if (message.data.wait) {
            willClearOutput = true;
          } else {
            this.state.clearOutput(cellId);
          }
        } else {
          console.warn("Skipping unhandled event", message);
        }
      };

      await invoke("run_cell", {
        kernelId: this.state.kernelId,
        code,
        onEvent,
      });
      if (status === "running") {
        status = "success";
      }
    } catch (error: any) {
      status = "error";
      // Synthesize an error output for kernel disconnects or other errors.
      this.state.appendOutput(cellId, {
        output_type: "error",
        ename: "InternalError",
        evalue: error.toString(),
        traceback: [],
      });
    } finally {
      timings = { ...timings, finishedAt: Date.now() };
      update();
    }
  }
}

/** Helper function to convert a maybe-multiline string to a string. */
function multiline(string: string | string[]): string {
  return typeof string === "string" ? string : string.join("");
}

export const NotebookContext = createContext<Notebook | undefined>(undefined);

export function useNotebook(): Notebook {
  const notebook = useContext(NotebookContext);
  if (!notebook) {
    throw new Error("useNotebook must be used within a NotebookContext");
  }
  return notebook;
}

import type { EditorView } from "@codemirror/view";
import { Channel, invoke } from "@tauri-apps/api/core";
import { encode } from "html-entities";
import { createContext, useContext } from "react";
import { v4 as uuidv4 } from "uuid";
import { StoreApi, createStore } from "zustand";
import { immer } from "zustand/middleware/immer";

import type { NotebookRoot, RunCellEvent } from "@/bindings";

type NotebookStore = NotebookStoreState & NotebookStoreActions;

/** Zustand reactive data used by the UI to render notebooks. */
export type NotebookStoreState = {
  /** A list of cell IDs in order. */
  cellIds: string[];

  /** Information about each cell, keyed by ID. */
  cells: {
    [cellId: string]: {
      type: CellType;
      initialText: string;
      output?: NotebookOutput;
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

export type NotebookOutput = {
  status: "success" | "error";
  output: string;
  timings: {
    startedAt: number;
    finishedAt?: number;
  };
  displays: { [displayId: string]: string };
};

/** Actions are kept private, only to be used from the `Notebook` class. */
type NotebookStoreActions = {
  /** Add a new cell to the notebook. */
  addCell: (id: string, type: CellType, initialText: string) => void;

  /** Set the type of a cell. */
  setCellType: (id: string, type: CellType) => void;

  /** Set the output of a cell after it runs. */
  setOutput: (cellId: string, output: NotebookOutput | undefined) => void;

  /**
   * Start loading the notebook from an external source.
   *
   * After this function is called, no new cell executions should happen until
   * the notebook finishes loading and one of the functions below is called. If
   * successful, this clears the current cells.
   */
  startLoading: () => void;

  /** Load the notebook from a JSON object. */
  loadNotebook: (notebook: NotebookRoot) => void;

  /** Set the error on failure to load a notebook. */
  setLoadError: (error: string) => void;

  /** Set the path of the notebook, when it is opened or saved. */
  setPath: (path: string) => void;
};

/** Initialize the Zustand store for a notebook and define mutators. */
function createNotebookStore(): StoreApi<NotebookStore> {
  return createStore<NotebookStore>()(
    immer<NotebookStore>((set) => ({
      cellIds: [],
      cells: {},
      isLoading: false,

      addCell: (cellId, type, initialText) =>
        set((state) => {
          state.cellIds.push(cellId);
          state.cells[cellId] = {
            type,
            initialText,
          };
        }),

      setCellType: (cellId, type) =>
        set((state) => {
          state.cells[cellId].type = type;
        }),

      setOutput: (cellId, output) =>
        set((state) => {
          state.cells[cellId].output = output;
        }),

      startLoading: () =>
        set((state) => {
          // TODO: Fix this to handle errors better.
          if (state.isLoading) throw new Error("Notebook is already loading");
          state.isLoading = true;
        }),

      loadNotebook: (notebook) =>
        set((state) => {
          // Filter out 'raw' cells, as they aren't supported yet.
          const cells = notebook.cells.filter(
            (cell) =>
              cell.cell_type === "code" || cell.cell_type === "markdown",
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

      setLoadError: (error) =>
        set((state) => {
          state.loadError = error;
          state.isLoading = false;
        }),

      setPath: (path) =>
        set((state) => {
          state.path = path;
        }),
    })),
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

  clearOutput(cellId: string) {
    this.state.setOutput(cellId, undefined);
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

    let status: NotebookOutput["status"] = "success";
    let output = "";
    let timings: NotebookOutput["timings"] = { startedAt: Date.now() };
    let displays: Record<string, any> = {};

    const update = () =>
      this.state.setOutput(cellId, {
        status,
        output,
        timings,
        displays,
      });
    update();

    try {
      const onEvent = new Channel<RunCellEvent>();

      onEvent.onmessage = (message: RunCellEvent) => {
        if (message.event === "stdout" || message.event === "stderr") {
          output += message.data;
          update();
        } else if (message.event === "error") {
          status = "error";
          output += `${message.data.ename}: ${message.data.evalue}\n`;
          update();
        } else if (message.event === "execute_result") {
          // This means that there was a return value for the cell.
          output += message.data.data["text/plain"];
          update();
        } else if (message.event === "display_data") {
          const displayId = message.data.transient?.display_id || uuidv4();
          const html = displayDataToHtml(
            message.data.data,
            message.data.metadata,
          );
          if (html) {
            displays = { ...displays, [displayId]: html };
            update();
          } else {
            console.warn("Skipping unhandled display data", message.data);
          }
        } else if (message.event === "update_display_data") {
          const displayId = message.data.transient?.display_id;
          if (displayId && Object.hasOwn(displays, displayId)) {
            const html = displayDataToHtml(
              message.data.data,
              message.data.metadata,
            );
            if (html) {
              displays = { ...displays, [displayId]: html };
              update();
            } else {
              console.warn("Skipping unhandled display data", message.data);
            }
          } else {
            console.warn("Skipping display for bad display ID", message.data);
          }
        } else if (message.event === "clear_output") {
          // TODO: Implement clear_output message type, with `wait=True` support.
          console.warn("clear_output not implemented yet");
        } else {
          console.warn("Skipping unhandled event", message);
        }
      };

      await invoke("run_cell", {
        kernelId: this.state.kernelId,
        code,
        onEvent,
      });
    } catch (error: any) {
      // TODO: Render backtraces properly here, and do not prune existing output.
      status = "error";
      output += error.toString();
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

/**
 * Returns the HTML form of a display data message.
 *
 * https://jupyter-client.readthedocs.io/en/stable/messaging.html#display-data
 */
function displayDataToHtml(
  data: Record<string, any>,
  metadata: Record<string, any>,
): string | null {
  for (const imageType of [
    "image/png",
    "image/jpeg",
    "image/svg+xml",
    "image/bmp",
    "image/gif",
  ]) {
    if (Object.hasOwn(data, imageType)) {
      const value = data[imageType];
      const alt = String(data["text/plain"] ?? "");
      const meta = metadata[imageType];
      if (typeof value === "string") {
        let image = `<img src="data:${imageType};base64,${encode(value)}" alt="${encode(alt)}"`;
        if (meta) {
          if (typeof meta.height === "number" && meta.height > 0) {
            image += ` height="${meta.height}"`;
          }
          if (typeof meta.width === "number" && meta.width > 0) {
            image += ` width="${meta.width}"`;
          }
        }
        image += " />";
        return image;
      }
    }
  }

  const value = data["text/plain"];
  if (typeof value === "string") {
    return `<pre>${encode(value)}</pre>`;
  }

  return null;
}

export const NotebookContext = createContext<Notebook | undefined>(undefined);

export function useNotebook(): Notebook {
  const notebook = useContext(NotebookContext);
  if (!notebook) {
    throw new Error("useNotebook must be used within a NotebookContext");
  }
  return notebook;
}

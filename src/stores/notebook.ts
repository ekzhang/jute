import type { EditorView } from "@codemirror/view";
import { Channel, invoke } from "@tauri-apps/api/core";
import { encode } from "html-entities";
import { createContext, useContext } from "react";
import { v4 as uuidv4 } from "uuid";
import { StoreApi, createStore } from "zustand";
import { immer } from "zustand/middleware/immer";

import type { Notebook as NotebookType, RunCellEvent } from "@/bindings";

type NotebookStore = NotebookStoreState & NotebookStoreActions;

export type NotebookStoreState = {
  cellIds: string[];
  cells: {
    [cellId: string]: {
      initialText: string;
      output?: NotebookOutput;
    };
  };

  /** ID of the running kernel, populated after the kernel is started. */
  kernelId?: string;

  /** True when loading the notebook from disk. */
  isLoading?: boolean;

  /** Error related to the notebook. */
  error?: string;
};

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
  addCell: (id: string, initialText: string) => void;
  setOutput: (cellId: string, output: NotebookOutput | undefined) => void;
};

type CellHandle = {
  editor?: EditorView;
};

export class Notebook {
  /** Promise that resolves when the kernel is started. */
  kernelStartPromise: Promise<void>;

  /** Zustand object used to reactively update DOM nodes. */
  store: StoreApi<NotebookStore>;

  /** Direct handles to editors and other HTML elements after render. */
  refs: Map<string, CellHandle>;

  /** The full path to the notebook. */
  path: string;

  /** The file name of the notebook. */
  filename: string;

  /** The directory of the notebook. */
  directory: string;

  constructor(path: string) {
    this.path = path;

    const parts = path.split("/");

    this.filename = parts.pop()!;
    this.directory = parts.join("/");

    this.store = createStore<NotebookStore>()(
      immer<NotebookStore>((set) => ({
        kernelId: undefined,
        isLoading: true,
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

    this.kernelStartPromise = this.startKernel();

    this.loadNotebook();
  }

  get state() {
    // Helper function, used internally to get the current notebook store state.
    return this.store.getState();
  }

  get kernelId() {
    return this.state.kernelId;
  }

  async loadNotebook() {
    try {
      const notebook = await invoke<NotebookType>("get_notebook", {
        path: this.path,
      });

      this.state.cellIds = notebook.cells.map((cell) => cell.id);

      this.state.cells = notebook.cells.reduce((acc, cell) => {
        this.refs.set(cell.id, {});
        acc[cell.id] = {
          initialText:
            typeof cell.source === "string"
              ? cell.source
              : cell.source.join("\n"),
          output: undefined,
        };
        return acc;
      }, this.state.cells);

      this.state.isLoading = false;
    } catch (e: unknown) {
      this.state.isLoading = false;

      if (e instanceof Error || typeof e === "string") {
        this.state.error = e.toString();
      } else {
        this.state.error =
          "An unknown error occurred while loading the notebook.";
      }
    }
  }

  async startKernel() {
    const kernelId = await invoke<string>("start_kernel", {
      specName: "python3",
    });
    this.store.setState({ kernelId });
  }

  addCell(initialText: string): string {
    const cellId = Math.random().toString(36).slice(2);
    this.refs.set(cellId, {});
    this.store.getState().addCell(cellId, initialText);
    return cellId;
  }

  clearOutput(cellId: string) {
    this.state.setOutput(cellId, undefined);
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
        } else {
          console.warn("Skipping unhandled event", message);
        }
      };

      await invoke("run_cell", { kernelId: this.kernelId, code, onEvent });
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

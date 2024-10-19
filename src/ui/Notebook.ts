import { EditorView } from "@codemirror/view";
import { invoke } from "@tauri-apps/api/core";
import { createContext } from "react";

export type NotebookOutput = { status: "success" | "error"; data: string };

export class Notebook {
  editors: Map<string, EditorView>;
  outputs: Map<string, NotebookOutput>;

  constructor(private rerender: () => void) {
    this.editors = new Map();
    this.outputs = new Map();
  }

  async execute(editorId: string) {
    const editor = this.editors.get(editorId);
    if (!editor) {
      throw new Error(`Editor ${editorId} not found`);
    }
    const code = editor.state.doc.toString();
    try {
      const output: string = await invoke("run_python", { sourceCode: code });
      this.outputs.set(editorId, { status: "success", data: output });
    } catch (error: any) {
      this.outputs.set(editorId, { status: "error", data: error });
    }
    this.rerender();
  }
}

export const NotebookContext = createContext<Notebook | undefined>(undefined);

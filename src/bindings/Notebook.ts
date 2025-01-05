// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { Cell } from "./Cell";
import type { NotebookMetadata } from "./NotebookMetadata";

/**
 * Represents the root structure of a Jupyter Notebook file.
 */
export type Notebook = {
  /**
   * Root-level metadata of the notebook.
   */
  metadata: NotebookMetadata;
  /**
   * Notebook format (minor number). Incremented for backward-compatible
   * changes.
   */
  nbformat_minor: number;
  /**
   * Notebook format (major number). Incremented for incompatible changes.
   */
  nbformat: number;
  /**
   * Array of cells in the notebook.
   */
  cells: Array<Cell>;
};

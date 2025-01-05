// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CellMetadata } from "./CellMetadata";
import type { MultilineString } from "./MultilineString";

/**
 * Raw cell in the notebook.
 */
export type RawCell = {
  /**
   * Identifier of the cell.
   */
  id: string;
  /**
   * Metadata for the cell.
   */
  metadata: CellMetadata;
  /**
   * Content of the cell.
   */
  source: MultilineString;
  /**
   * Attachments (e.g., images) in the cell.
   */
  attachments:
    | { [key in string]?: { [key in string]?: MultilineString } }
    | null;
};

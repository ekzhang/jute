// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { CodeCell } from "./CodeCell";
import type { MarkdownCell } from "./MarkdownCell";
import type { RawCell } from "./RawCell";

/**
 * Represents a notebook cell, which can be a raw, markdown, or code cell.
 */
export type Cell =
  | ({ cell_type: "raw" } & RawCell)
  | ({ cell_type: "markdown" } & MarkdownCell)
  | ({ cell_type: "code" } & CodeCell);

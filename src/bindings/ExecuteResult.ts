// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { JsonBTreeMap } from "./JsonBTreeMap";
import type { MultilineString } from "./MultilineString";

/**
 * Result of executing a code cell.
 */
export type ExecuteResult = { 
/**
 * Execution count of the result.
 */
execution_count: number | null, 
/**
 * Data returned by the execution.
 */
data: { [key in string]?: MultilineString }, 
/**
 * Metadata associated with the result.
 */
metadata: JsonBTreeMap, };
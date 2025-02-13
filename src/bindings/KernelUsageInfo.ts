// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Contains information about the CPU and memory usage of a kernel.
 */
export type KernelUsageInfo = {
  /**
   * Number of CPUs used.
   */
  cpu_consumed: number;
  /**
   * Number of CPUs available.
   */
  cpu_available: number;
  /**
   * Memory consumed in KB.
   */
  memory_consumed: number;
  /**
   * Memory available in KB.
   */
  memory_available: number;
};

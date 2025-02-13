// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

/**
 * Content of an error response message.
 */
export type ErrorReply = {
  /**
   * The error name, such as 'NameError'.
   */
  ename: string;
  /**
   * The error message, such as 'NameError: name 'x' is not defined'.
   */
  evalue: string;
  /**
   * The traceback frames of the error as a list of strings.
   */
  traceback: Array<string>;
};

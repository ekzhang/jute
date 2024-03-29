import clsx from "clsx";

import { NotebookOutput } from "./Notebook";

export type OutputViewProps = {
  value: NotebookOutput | undefined;
};

export default ({ value }: OutputViewProps) => {
  if (!value) {
    return null;
  }
  return (
    <pre
      className={clsx(
        value.status === "error" && "text-red-500",
        "select-text whitespace-pre-wrap break-words text-sm",
      )}
    >
      {value.data}
    </pre>
  );
};

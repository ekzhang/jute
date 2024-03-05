import { CircleIcon } from "lucide-react";
import { useContext } from "react";

import CmEditor from "./CmEditor";
import { NotebookContext } from "./Notebook";
import OutputView from "./OutputView";

export default () => {
  const notebook = useContext(NotebookContext)!;

  const editorIds = ["1", "2"];

  return (
    <div className="py-16">
      {editorIds.map((id) => (
        <div key={id}>
          <div className="flex items-start gap-2">
            <CircleIcon className="my-[6px] h-3.5 w-3.5 fill-zinc-300 stroke-none" />
            <div className="flex-1">
              <CmEditor editorId={id} />
            </div>
          </div>
          {notebook.outputs.get(id) && (
            <div className="px-6 pt-4">
              <OutputView value={notebook.outputs.get(id)} />
            </div>
          )}
          <hr className="mx-2 my-4 border-zinc-200" />
        </div>
      ))}
    </div>
  );
};

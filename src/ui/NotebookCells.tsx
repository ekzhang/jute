import { CircleIcon, CornerDownRightIcon } from "lucide-react";
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
          <div className="flex items-start gap-2 px-4">
            <CircleIcon className="my-[6px] h-3.5 w-3.5 fill-zinc-200 stroke-none" />
            <div className="flex-1">
              <CmEditor editorId={id} />
            </div>
          </div>
          {notebook.outputs.get(id) && (
            <div className="flex gap-3 px-4 pt-4">
              <div>
                <CornerDownRightIcon className="ml-1 h-5 w-5 text-green-700" />
              </div>
              <div className="pt-0.5">
                <OutputView value={notebook.outputs.get(id)} />
              </div>
            </div>
          )}
          <hr className="mx-2 my-4 border-zinc-200" />
        </div>
      ))}
    </div>
  );
};

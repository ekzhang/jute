import {
  BoltIcon,
  CheckIcon,
  Code2Icon,
  LucideIcon,
  PlusIcon,
  RouteOffIcon,
  XIcon,
  XSquareIcon,
} from "lucide-react";
import { ReactNode, Suspense, lazy } from "react";
import { useStore } from "zustand";
import CellInputFallback from "~/components/cells/CellInputFallback";
import OutputView from "~/components/cells/OutputView";
import { useNotebook } from "~/hooks/notebook";

const CellInput = lazy(() => import("../cells/CellInput"));

const Aside = ({ children }: { children: ReactNode }) => (
  <aside className="absolute right-[-200px] w-[200px] px-2">{children}</aside>
);

const AsideIconButton = ({
  Icon,
  onClick,
}: {
  Icon: LucideIcon;
  onClick?: () => void;
}) => (
  <button
    className="rounded p-1 text-gray-500 transition-all hover:bg-gray-200 hover:text-black active:scale-110"
    onClick={onClick}
  >
    <Icon size={16} />
  </button>
);

function CellInputAside({ cellId }: { cellId: string }) {
  const notebook = useNotebook();
  const output = useStore(
    notebook.store,
    (state) => state.cells[cellId].output,
  );

  // TODO: Real-time clock indicator here.
  return (
    <Aside>
      <div className="mt-1 flex gap-0.5">
        <AsideIconButton Icon={Code2Icon} />
        <AsideIconButton Icon={RouteOffIcon} />
        <AsideIconButton Icon={BoltIcon} />
      </div>
      {output?.timings?.finishedAt && (
        <div className="mt-0.5 flex items-center">
          {output.status === "success" ? (
            <CheckIcon size={16} className="mr-1 text-green-500" />
          ) : (
            <XIcon size={16} className="mr-1 text-red-500" />
          )}
          <p className="text-sm text-gray-400">
            {output?.timings.finishedAt - output?.timings.startedAt} ms
          </p>
        </div>
      )}
    </Aside>
  );
}

export default function NotebookCells() {
  const notebook = useNotebook();
  const cellIds = useStore(notebook.store, (state) => state.cellIds);
  const cells = useStore(notebook.store, (state) => state.cells);

  return (
    <div className="relative py-8">
      {cellIds.map((id) => (
        <div key={id}>
          <hr className="border-gray-200" />

          <CellInputAside cellId={id} />
          <Suspense fallback={<CellInputFallback cellId={id} />}>
            <CellInput cellId={id} />
          </Suspense>

          {cells[id]?.output && (
            <>
              <hr className="border-gray-200" />
              <Aside>
                <div className="mt-1 flex gap-0.5">
                  <AsideIconButton
                    Icon={XSquareIcon}
                    onClick={() => notebook.clearOutput(id)}
                  />
                  <AsideIconButton Icon={BoltIcon} />
                </div>
              </Aside>
              <div className="max-h-[560px] overflow-y-auto">
                {/* TODO: Move this icon into the output view itself. Also it should only be displayed
                  when the cell has a return value, and next to the return value. */}
                {/* <CornerDownRightIcon size={16} className="text-gray-400" /> */}
                <OutputView value={cells[id].output} />
              </div>
            </>
          )}
        </div>
      ))}

      <div className="mx-2 my-4">
        <button
          className="flex w-full items-center justify-center gap-1.5 rounded border border-gray-200 p-2 transition-colors hover:border-gray-300 hover:bg-gray-50"
          onClick={() => {
            notebook.addCell("");
          }}
        >
          <PlusIcon size={18} />
          <span>New cell</span>
        </button>
      </div>
    </div>
  );
}

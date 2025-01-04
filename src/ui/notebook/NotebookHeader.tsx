import clsx from "clsx";
import {
  ChartLineIcon,
  DotIcon,
  HomeIcon,
  PlayIcon,
  PlusIcon,
  RefreshCwIcon,
  SettingsIcon,
} from "lucide-react";
import { Link } from "wouter";
import { useStore } from "zustand";

import { useNotebook } from "@/stores/notebook";

type Props = {
  kernelName: string;
};

export default function NotebookHeader({ kernelName }: Props) {
  const notebook = useNotebook();

  const kernelId = useStore(notebook.store, (state) => state.kernelId);

  return (
    <div className="absolute inset-x-0 h-16 bg-gradient-to-b from-white/85 from-40% to-white/0">
      <header
        className="flex h-[38px] items-center justify-between gap-6 px-3"
        data-tauri-drag-region
      >
        {/* Empty placeholder to take up space where the traffic light buttons are. */}
        <div className="w-16" />

        {/* Centered UI components: kernel controls and stats. */}
        <div className="flex items-center">
          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <PlayIcon size={16} />
          </button>
          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <RefreshCwIcon size={16} />
          </button>

          <button className="mx-2 flex w-60 items-center justify-center rounded border border-gray-200 py-[3px] text-xs text-gray-900 transition-all hover:border-gray-400 hover:bg-gray-100 active:scale-105">
            <DotIcon
              className={clsx(
                "w-6",
                kernelId === "" ? "text-orange-500" : "text-green-500",
              )}
            />
            {kernelName}
            <div className="w-6" />{" "}
            {/* Same width as the icon to make sure kernel name is center aligned */}
          </button>

          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <ChartLineIcon size={16} />
          </button>
        </div>

        {/* Top-right UI components: settings and open notebooks. */}
        <div className="flex items-center">
          <Link to="/">
            <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
              <HomeIcon size={20} strokeWidth={1.5} />
            </button>
          </Link>
          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <SettingsIcon size={20} strokeWidth={1.5} />
          </button>
          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <PlusIcon size={20} strokeWidth={1.5} />
          </button>
        </div>
      </header>
    </div>
  );
}

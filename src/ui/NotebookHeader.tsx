import {
  ChartLineIcon,
  PlayIcon,
  PlusIcon,
  RefreshCwIcon,
  SettingsIcon,
} from "lucide-react";

type Props = {
  kernelName: string;
};

export default function NotebookHeader({ kernelName }: Props) {
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

          <button className="mx-2 w-60 rounded border border-gray-200 py-[3px] text-xs text-gray-900 transition-all hover:border-gray-400 hover:bg-gray-100 active:scale-105">
            {kernelName}
          </button>

          <button className="rounded p-1 text-gray-500 transition-all hover:bg-gray-100 hover:text-black active:scale-110">
            <ChartLineIcon size={16} />
          </button>
        </div>

        {/* Top-right UI components: settings and open notebooks. */}
        <div className="flex items-center">
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

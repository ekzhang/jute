import { invoke } from "@tauri-apps/api/core";
import { LucideIcon, RouteIcon, SparklesIcon } from "lucide-react";
import { useEffect, useState } from "react";
import { ErrorBoundary } from "react-error-boundary";
import { useStore } from "zustand";

import { KernelUsageInfo } from "@/bindings";
import { useNotebook } from "@/stores/notebook";

const FeatureButton = ({
  title,
  Icon,
  onClick,
}: {
  title: string;
  Icon: LucideIcon;
  onClick?: () => void;
}) => (
  <button
    className="flex items-center rounded px-1.5 py-0.5 text-gray-500 transition-colors hover:bg-gray-100 hover:text-black"
    onClick={onClick}
  >
    <Icon size={16} className="mr-1.5" />
    <span className="text-sm">{title}</span>
  </button>
);

const round = (n: number, precision: number = 0) => {
  const factor = Math.pow(10, precision);
  return Math.round(n * factor) / factor;
};

const formatMemory = (bytes: number) => {
  if (bytes < 1024) return `${round(bytes)} B`;
  if (bytes < 1024 * 1024) return `${round(bytes / 1024)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${round(bytes / (1024 * 1024), 1)} MB`;
  return `${round(bytes / (1024 * 1024 * 1024), 2)} GB`;
};

const KernelUsage = () => {
  const notebook = useNotebook();

  const [cpu, setCpu] = useState(0);
  const [cpuAvailable, setCpuAvailable] = useState(0);
  const [mem, setMem] = useState(0);
  const [memAvailable, setMemAvailable] = useState(0);
  const kernelId = useStore(notebook.store, (state) => state.kernelId);

  useEffect(() => {
    const updateUsage = async () => {
      if (!kernelId) return;

      const usageInfo = (await invoke("kernel_usage_info", {
        kernelId,
      })) as KernelUsageInfo;

      setCpu(usageInfo.cpu_consumed);
      setCpuAvailable(usageInfo.cpu_available);
      setMem(usageInfo.memory_consumed);
      setMemAvailable(usageInfo.memory_available);
    };

    // fetch immediately then every 5s
    updateUsage();
    const timer = setInterval(async () => {
      updateUsage();
    }, 5000);

    return () => clearInterval(timer);
  }, [kernelId]);

  const cpuUsage = cpuAvailable > 0 ? (cpu / cpuAvailable) * 100 : undefined;
  const memoryUsage = memAvailable > 0 ? (mem / memAvailable) * 100 : undefined;

  return (
    <div className="ml-auto flex items-center">
      {memoryUsage != undefined && (
        <>
          <p className="cursor-default text-sm text-gray-500">RAM</p>
          <div
            className="ml-2 h-2 w-20 overflow-hidden rounded-full bg-gray-300"
            title={`${formatMemory(mem)} / ${formatMemory(memAvailable)} memory used`}
          >
            <div
              className="h-full bg-green-600 transition-[width]"
              style={{ width: memoryUsage + "%", transition: "width 0.5s" }}
            />
          </div>
        </>
      )}

      {cpuUsage !== undefined && (
        <>
          <p className="ml-4 cursor-default text-sm text-gray-500">CPU</p>
          <div
            className="ml-2 h-2 w-20 overflow-hidden rounded-full bg-gray-300"
            title={`${round(cpu, 1)} / ${round(cpuAvailable, 1)} cores used`}
          >
            <div
              className="h-full bg-pink-600 transition-[width]"
              style={{ width: cpuUsage + "%", transition: "width 0.5s" }}
            />
          </div>
        </>
      )}
    </div>
  );
};

export default function NotebookFooter() {
  return (
    <div className="absolute inset-x-0 bottom-0 z-10 flex h-16 flex-col justify-end bg-gradient-to-t from-white/85 from-40% to-white/0">
      <footer className="flex items-end gap-6 px-2 py-1">
        <div className="flex items-center gap-1">
          <FeatureButton title="AI Copilot" Icon={SparklesIcon} />
          <div className="h-1 w-1 rounded-full bg-gray-300" />
          <FeatureButton title="Reactivity" Icon={RouteIcon} />
        </div>

        <ErrorBoundary fallback={undefined}>
          <KernelUsage />
        </ErrorBoundary>
      </footer>
    </div>
  );
}

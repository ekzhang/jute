import { LucideIcon, RouteIcon, SparklesIcon } from "lucide-react";

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

export default function NotebookFooter() {
  return (
    <div className="absolute inset-x-0 bottom-0 flex h-16 flex-col justify-end bg-gradient-to-t from-white/85 from-40% to-white/0">
      <footer className="flex items-end gap-6 px-2 py-1">
        <div className="flex items-center gap-1">
          <FeatureButton title="AI Copilot" Icon={SparklesIcon} />
          <div className="h-1 w-1 rounded-full bg-gray-300" />
          <FeatureButton title="Reactivity" Icon={RouteIcon} />
        </div>

        <div className="ml-auto flex items-center">
          <p className="cursor-default text-sm text-gray-500">RAM</p>
          <div className="ml-2 h-2 w-20 overflow-hidden rounded-full bg-gray-300">
            <div
              className="h-full bg-green-600 transition-[width]"
              style={{ width: "55%" }}
            />
          </div>

          <p className="ml-4 cursor-default text-sm text-gray-500">CPU</p>
          <div className="ml-2 h-2 w-20 overflow-hidden rounded-full bg-gray-300">
            <div
              className="h-full bg-pink-600 transition-[width]"
              style={{ width: "12%" }}
            />
          </div>
        </div>
      </footer>
    </div>
  );
}

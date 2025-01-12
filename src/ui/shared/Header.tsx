import { ReactNode } from "react";

type Props = {
  children?: ReactNode;
};

export default function Header({ children }: Props) {
  return (
    <div className="absolute inset-x-0 h-16 bg-gradient-to-b from-white/85 from-40% to-white/0">
      <header
        className="flex h-[38px] items-center justify-between gap-6 px-3"
        data-tauri-drag-region
      >
        {children}
      </header>
    </div>
  );
}

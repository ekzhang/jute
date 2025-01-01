export type TitleBarProps = {
  title: string;
};

export default ({ title }: TitleBarProps) => {
  return (
    <div
      className="absolute inset-x-0 z-10 bg-gradient-to-b from-white/75 from-60% to-white/0 pb-3 pt-1 text-center text-sm"
      data-tauri-drag-region
    >
      <span className="pointer-events-none">{title}</span>
    </div>
  );
};

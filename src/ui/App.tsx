import NotebookView from "./NotebookView";

export default () => {
  const title = "sediment_analysis.ipynb";

  return (
    <main className="absolute inset-0 bg-white">
      {/* Title bar */}
      <div
        className="absolute inset-x-0 z-10 bg-gradient-to-b from-white/75 from-60% to-white/0 pb-3 pt-1 text-center text-sm"
        data-tauri-drag-region
      >
        <span className="pointer-events-none">{title}</span>
      </div>

      <NotebookView />
    </main>
  );
};

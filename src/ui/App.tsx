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

      {/* Main notebook */}
      <div className="grid h-full overflow-y-auto md:grid-cols-[600px,minmax(0,1fr)] lg:grid-cols-[800px,minmax(0,1fr)]">
        <div className="px-4 py-16">
          <NotebookView />
        </div>
        <div
          className="hidden border-l border-zinc-200 bg-zinc-100 md:block"
          data-tauri-drag-region
        >
          <p></p>
        </div>
      </div>
    </main>
  );
};

import NotebookView from "./NotebookView";

export default () => {
  const title = "sediment_analysis.ipynb";

  return (
    <main className="absolute inset-0 bg-white">
      {/* Title bar */}
      <div className="absolute inset-x-0 z-10 bg-gradient-to-b from-white/75 from-60% to-white/0 pb-5 pt-1 text-center text-sm">
        {title}
      </div>

      {/* Main notebook */}
      <div className="grid h-full overflow-y-auto sm:grid-cols-[minmax(0,3fr),minmax(0,1fr)]">
        <div className="max-w-screen-md px-4 py-16">
          <NotebookView />
        </div>
        <div
          className="hidden border-l border-zinc-200 bg-zinc-100 sm:block"
          data-tauri-drag-region
        >
          <p>test</p>
        </div>
      </div>
    </main>
  );
};

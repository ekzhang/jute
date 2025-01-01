import NotebookView from "./NotebookView";
import TitleBar from "./TitleBar";

export default () => {
  const title = "sediment_analysis.ipynb";

  return (
    <main className="absolute inset-0 bg-white">
      <TitleBar title={title} />
      <NotebookView />
    </main>
  );
};

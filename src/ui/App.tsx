import NotebookFooter from "./NotebookFooter";
import NotebookHeader from "./NotebookHeader";
import NotebookView from "./NotebookView";

export default function App() {
  return (
    <main className="absolute inset-0 bg-white">
      <NotebookHeader kernelName="Local Kernel (Python 3.11.7)" />
      <NotebookView />
      <NotebookFooter />
    </main>
  );
}

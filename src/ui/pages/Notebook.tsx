import NotebookFooter from "~/components/notebooks/NotebookFooter";
import NotebookHeader from "~/components/notebooks/NotebookHeader";
import NotebookView from "~/components/notebooks/NotebookView";

export default function Notebook() {
  return (
    <>
      <NotebookHeader kernelName="Local Kernel (Python 3.11.7)" />
      <NotebookView />
      <NotebookFooter />
    </>
  );
}

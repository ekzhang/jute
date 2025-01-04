import "@fontsource-variable/fira-code";
import { invoke } from "@tauri-apps/api/core";
import { createRoot } from "react-dom/client";

import App from "./ui/App";

if (import.meta.env.MODE === "development") {
  // For development purposes, save the invoke() function in global scope.
  (window as any).invoke = invoke;
}

if ((window as any).__jute_opened_file) {
  // TODO: Maybe investigate putting this into the URL path as a hash instead of
  // injecting it as the `window.__jute_opened_files` global. We could use a
  // HashRouter with like `app://index.html#/notebook?path=/path/to/file.ipynb`
  // and construct this URL in the Rust code that builds a window.
  console.log("__jute_opened_file =", (window as any).__jute_opened_file);
}

const root = createRoot(document.getElementById("root")!);
root.render(<App openedFile={(window as any).__jute_opened_file} />);

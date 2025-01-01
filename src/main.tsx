import "@fontsource-variable/fira-code";
import { invoke } from "@tauri-apps/api/core";
import { createRoot } from "react-dom/client";

import App from "./ui/App";

if (import.meta.env.MODE === "development") {
  // For development purposes, save the invoke() function in global scope.
  (window as any).invoke = invoke;
}

const root = createRoot(document.getElementById("root")!);
root.render(<App />);

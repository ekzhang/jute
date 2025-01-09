import { ErrorBoundary, type FallbackProps } from "react-error-boundary";
import { Route, Switch } from "wouter";

import HomePage from "@/pages/HomePage";
import NotFoundPage from "@/pages/NotFoundPage";
import NotebookPage from "@/pages/NotebookPage";

import { UnhandledError } from "./ui/shared/UnhandledError";

function fallbackRender({ error, resetErrorBoundary }: FallbackProps) {
  return (
    <div className="h-screen">
      <UnhandledError error={error.toString()} onReturn={resetErrorBoundary} />
    </div>
  );
}

export default function App() {
  return (
    <ErrorBoundary fallbackRender={fallbackRender}>
      <Switch>
        <Route path="/" component={HomePage} />
        <Route path="/notebook" component={NotebookPage} />

        <Route component={NotFoundPage} />
      </Switch>
    </ErrorBoundary>
  );
}

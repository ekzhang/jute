import { useEffect } from "react";
import { Route, Router, Switch } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";

import HomePage from "@/pages/HomePage";
import NotebookPage from "@/pages/NotebookPage";

type Props = {
  openedFile?: string;
};

export default function App({ openedFile }: Props) {
  const [, navigate] = useHashLocation();

  // if we're opened with a file, we should navigate to the notebook page
  useEffect(() => {
    if (openedFile) {
      navigate(`/notebook/${encodeURIComponent(openedFile)}`);
    }
  });

  return (
    <Router hook={useHashLocation}>
      <Switch>
        <Route path="/" component={HomePage} />
        <Route path="/notebook/:encodedPath" component={NotebookPage} />
      </Switch>
    </Router>
  );
}

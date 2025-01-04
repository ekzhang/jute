import { useEffect } from "react";
import { Route, Router, Switch } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";

import Home from "~/pages/Home";
import Notebook from "~/pages/Notebook";

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
    <main className="absolute inset-0 bg-white">
      <Router hook={useHashLocation}>
        <Switch>
          <Route path="/">
            <Home />
          </Route>

          <Route path="/notebook/:encodedPath">
            <Notebook />
          </Route>
        </Switch>
      </Router>
    </main>
  );
}

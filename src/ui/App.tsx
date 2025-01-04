import { Route, Router, Switch } from "wouter";
import { useHashLocation } from "wouter/use-hash-location";

import Home from "~/pages/Home";
import Notebook from "~/pages/Notebook";

export default function App() {
  return (
    <main className="absolute inset-0 bg-white">
      <Router hook={useHashLocation}>
        <Switch>
          <Route path="/">
            <Home />
          </Route>
          <Route path="/notebook/:path">
            <Notebook />
          </Route>
        </Switch>
      </Router>
    </main>
  );
}

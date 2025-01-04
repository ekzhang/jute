import { Route, Switch } from "wouter";
import Home from "~/pages/Home";
import Notebook from "~/pages/Notebook";

export default function App() {
  return (
    <main className="absolute inset-0 bg-white">
      <Switch>
        <Route path="/">
          <Home />
        </Route>
        <Route path="/notebook/:path">
          <Notebook />
        </Route>
      </Switch>
    </main>
  );
}

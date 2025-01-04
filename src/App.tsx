import { Route, Switch } from "wouter";

import HomePage from "@/pages/HomePage";
import NotebookPage from "@/pages/NotebookPage";

export default function App() {
  return (
    <Switch>
      <Route path="/" component={HomePage} />
      <Route path="/notebook" component={NotebookPage} />
    </Switch>
  );
}

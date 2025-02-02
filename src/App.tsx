import { Route, Router } from "@solidjs/router";
import "./App.css";
import PluginSelect from "./pages/plugin-select";
import Folders from "./pages/folders";

function App() {
  return <Router>
    <Route path={"/folders"} component={Folders} />
    <Route path={"*"} component={PluginSelect} />
  </Router>
}

export default App;

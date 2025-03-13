import { Route, Router } from "@solidjs/router";
import "./App.css";
import PluginSelect from "./pages/plugin-select";
import Fmap from "./pages/fmap";
import ErrorPage from "./pages/error-page";
import Mapping from "./pages/mapping";
import { Window } from "@tauri-apps/api/window";
import { listen } from "@/logic/backend";
import { createWindow } from "@/logic/window";
import { Toaster } from "solid-toast";
import Conflicting from "./pages/conflicting";

(() => {
  const parent = Window.getCurrent()
  if (parent.label == "main")
    listen("plugin_error", ([title, description]) => createWindow(`/error/${description}`, { title, parent }))
})()

function App() {
  return <>
    <Toaster position="bottom-left" />
    <Router>
      <Route path={"/folders"} component={Fmap} />
      <Route path={"/error/*ERROR"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"/conflicting/FOLDERNAME/*TAG"} component={Conflicting} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </>
}

export default App;

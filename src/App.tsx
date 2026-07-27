import "./App.css";
import { createStore } from "solid-js/store";
import { Route, Router } from "@solidjs/router";
import { Window } from "@tauri-apps/api/window";
import { listen } from "@/logic/backend";
import { createWindow } from "@/logic/window";
import { Toaster } from "solid-toast";
import PageRoot from "./PageRoot";
import { FileTree } from "./types/data";
import { Conflicting, ErrorPage, Folders, Mapping, PluginSelect, Tags } from "./pages";

export const [folders, setFolders] = createStore<FileTree>();

(() => {
  const parent = Window.getCurrent()
  listen("plugin_error", ([title, description]) => createWindow(`/error/${description}`, { title, parent }))
})()

function App() {
  return <>
    <Toaster position="bottom-center" containerClassName="cursor-pointer" />
    <Router root={PageRoot}>
      <Route path={"/tags"} component={Tags} />
      <Route path={"/tags/:TAGNAME"} component={Folders} />
      <Route path={"/error/*ERROR"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"/conflicting/:FOLDERNAME/:LOCAL/:CLOUD/*TAG"} component={Conflicting} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </>
}

export default App;

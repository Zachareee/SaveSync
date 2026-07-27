import "./App.css";
import { createStore } from "solid-js/store";
import { Route, Router } from "@solidjs/router";
import { listen } from "@/logic/backend";
import { createWindow } from "@/logic/window";
import { Toaster } from "solid-toast";
import PageRoot from "./PageRoot";
import { FileTree } from "./types/data";
import { Conflicting, ErrorPage, Folders, Mapping, PluginSelect, Tags, Settings } from "./pages";

export const [folders, setFolders] = createStore<FileTree>();

(() => {
  listen("plugin_error", ([title, error]) => createWindow(`/error?${{ error }}`, { title, parent: "main" }))
})()

function App() {
  return <>
    <Toaster position="bottom-center" containerClassName="cursor-pointer" />
    <Router root={PageRoot}>
      <Route path={"/tags"} component={Tags} />
      <Route path={"/tags/:TAGNAME"} component={Folders} />
      <Route path={"/error"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"/conflicting"} component={Conflicting} />
      <Route path={"/settings"} component={Settings} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </>
}

export default App;

import { Route, Router } from "@solidjs/router";
import "./App.css";
import PluginSelect from "./pages/plugin-select";
import Folders from "./pages/folders";
import Fmap from "./pages/fmap";
import ErrorPage from "./pages/error-page";
import { createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { FolderMapping } from "@/types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "./utils";

listen("plugin_error", ({ payload: [title, description] }) => new WebviewWindow(title, { focus: true, visible: true, title, url: `/error/${description}` }).once("tauri://error", console.log))

function App() {
  const [folders, setFolders] = createStore<FolderMapping>()
  return <FolderContext.Provider value={{ folders, setFolders }}>
    <Router>
      <Route path={"/folders"} component={Fmap} />
      <Route path={"/folders/:TAG"} component={Folders} />
      <Route path={"/error/:ERROR"} component={ErrorPage} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </FolderContext.Provider>
}

type Stores = {
  folders: FolderMapping
}

type StoresWithSetters = Stores & {
  [k in keyof Stores as `set${Capitalize<k>}`]: ReturnType<typeof createStore<Stores[k]>>[1]
}

export default App;
const FolderContext = createContext<StoresWithSetters>()
export function useFolderContext() {
  return useContext(FolderContext)
}

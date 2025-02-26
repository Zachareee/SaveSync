import { Route, Router } from "@solidjs/router";
import "./App.css";
import PluginSelect from "./pages/plugin-select";
import Folders from "./pages/folders";
import Fmap from "./pages/fmap";
import ErrorPage from "./pages/error-page";
import { createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { FileTree } from "@/types";
import Mapping from "./pages/mapping";
import { Window } from "@tauri-apps/api/window";
import { listen } from "@/logic/backend";
import { createWindow } from "./logic/window";

(() => {
  if (Window.getCurrent().label == "main") listen("plugin_error", ({ payload: [title, description] }) => createWindow(title, { url: `/error/${description}` }))
})()

function App() {
  const [folders, setFolders] = createStore<FileTree>()
  return <FolderContext.Provider value={{ folders, setFolders }}>
    <Router>
      <Route path={"/folders"} component={Fmap} />
      <Route path={"/folders/:TAG"} component={Folders} />
      <Route path={"/error/:ERROR"} component={ErrorPage} />
      <Route path={"/mapping"} component={Mapping} />
      <Route path={"*"} component={PluginSelect} />
    </Router>
  </FolderContext.Provider>
}


type Stores = {
  folders: FileTree
}

type StoresWithSetters = Stores & {
  [k in keyof Stores as `set${Capitalize<k>}`]: ReturnType<typeof createStore<Stores[k]>>[1]
}

export default App;
const FolderContext = createContext<StoresWithSetters>()
export function useFolderContext() {
  return useContext(FolderContext)
}

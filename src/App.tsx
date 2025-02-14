import { Route, Router } from "@solidjs/router";
import "./App.css";
import PluginSelect from "./pages/plugin-select";
import Folders from "./pages/folders";
import Fmap from "./pages/fmap";
import { createContext, useContext } from "solid-js";
import { createStore } from "solid-js/store";
import { FolderMapping } from "@/types";

function App() {
  const [folders, setFolders] = createStore<FolderMapping>()
  return <FolderContext.Provider value={{ folders, setFolders }}>
    <Router>
      <Route path={"/folders"} component={Fmap} />
      <Route path={"/folders/:TAG"} component={Folders} />
      <Route path={"/error/:ERROR"} component={() => <div>Help</div>} />
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

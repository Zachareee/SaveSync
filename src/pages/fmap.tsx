import { useFolderContext } from "@/App"
import { emit, invoke, listen } from "@/logic/backend"
import { useNavigate } from "@solidjs/router"
import { Index, onCleanup } from "solid-js"
import { menuStatus } from "@/logic/menu"
import toast from "solid-toast"

export default function Fmap() {
  menuStatus(true)

  const navigate = useNavigate()
  const { folders, setFolders } = useFolderContext()!

  invoke("get_filetree").then(setFolders)
  const unlisten = listen("ignored_tags", () => {
    toast.error("Some folders were not synced, please check File -> Mappings")
  })

  onCleanup(async () => (await unlisten)())

  return <main class="container">
    <Index each={Object.entries(folders)}>
      {elem => <div class="border-white m-4" onclick={[navigate, elem()[0]]}>
        <p>{elem()[0]}</p>
      </div>}
    </Index>
    <div>
      <button onclick={() => { emit("unload"); navigate("/") }}>Back to plugin select</button>
    </div>
  </main>
}

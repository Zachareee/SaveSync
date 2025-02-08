import { useFolderContext } from "@/App"
import { invoke } from "@/utils"
import { useNavigate } from "@solidjs/router"
import { Index } from "solid-js"

export default function Fmap() {
  const navigate = useNavigate()
  const { folders, setFolders } = useFolderContext()!

  invoke("get_fmap").then(setFolders)

  return <main class="container">
    <Index each={Object.entries(folders)}>
      {elem => <div class="border-white m-4" onclick={[navigate, elem()[0]]}>
        <p>{elem()[0]}</p>
      </div>}
    </Index><div>
      <button onclick={() => navigate("/")}>Back to plugin select</button>
    </div>
  </main>
}


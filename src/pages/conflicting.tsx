import { emit } from "@/logic/backend"
import { useParams } from "@solidjs/router"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"


export default function Conflicting() {
  const { FOLDERNAME, TAG, LOCAL, CLOUD } = useParams()

  const reply = (reply: string) => {
    emit("conflict_resolve", [TAG, FOLDERNAME, reply])
    getCurrentWebviewWindow().close()
  }

  return <>
    <h1>The folder {FOLDERNAME} in tag {TAG} from the cloud might overwrite unsaved work</h1>
    <h2>Which would you like to keep?</h2>
    <div>
      <button onclick={[reply, "local"]}>
        Local files
        {new Date(LOCAL).toString()}
      </button>
      <button onclick={[reply, "cloud"]}>
        Cloud files
        {new Date(CLOUD).toString()}
      </button>
      <button onclick={[reply, "none"]}>Let me decide</button>
    </div>
  </>
}

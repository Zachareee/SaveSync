import { dateToLocaleString, emit, stringToOsString } from "@/logic/backend"
import { useParams } from "@solidjs/router"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"


export default function Conflicting() {
  const { FOLDERNAME, TAG, LOCAL, CLOUD } = useParams()

  const reply = (reply: string) => {
    emit("conflict_resolve", [TAG, stringToOsString(FOLDERNAME), reply])
    getCurrentWebviewWindow().close()
  }

  return <main class="container">
    <h1>The folder {FOLDERNAME} in tag {TAG} from the cloud might overwrite unsaved work</h1>
    <h2>Which would you like to keep?</h2>
    <br />
    <div class="space-x-2 flex items-stretch justify-center">
      <button onclick={[reply, "local"]}>
        Local files
        <br />
        {dateToLocaleString(new Date(parseInt(LOCAL) * 1000))}
      </button>
      <button onclick={[reply, "cloud"]}>
        Cloud files
        <br />
        {dateToLocaleString(new Date(parseInt(CLOUD) * 1000))}
      </button>
      <button onclick={[reply, "none"]}>Let me decide</button>
    </div>
  </main>
}

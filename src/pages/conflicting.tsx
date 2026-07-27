import { dateToLocaleString, emit, stringToOsString } from "@/logic/backend"
import { useSearchParams } from "@solidjs/router"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"

export default function Conflicting() {
  const [params, _setter] = useSearchParams<Record<"tag" | "folder" | "local" | "cloud", string>>()
  const { tag, folder, local, cloud } = params as Required<typeof params>

  const reply = (reply: string) => {
    emit("conflict_resolve", [tag, stringToOsString(folder), reply])
    getCurrentWebviewWindow().close()
  }

  return <main class="container">
    <h1>The folder {folder} in tag {tag} from the cloud might overwrite unsaved work</h1>
    <h2>Which would you like to keep?</h2>
    <br />
    <div class="space-x-2 flex items-stretch justify-center">
      <button onclick={[reply, "local"]}>
        Local files
        <br />
        {dateToLocaleString(new Date(parseInt(local) * 1000))}
      </button>
      <button onclick={[reply, "cloud"]}>
        Cloud files
        <br />
        {dateToLocaleString(new Date(parseInt(cloud) * 1000))}
      </button>
      <button onclick={[reply, "none"]}>Let me decide</button>
    </div>
  </main>
}

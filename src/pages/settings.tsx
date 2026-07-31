import PageRoot from "@/PageRoot"
import Switch from "@suid/material/Switch"
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart"
import { load } from '@tauri-apps/plugin-store'
import { createSignal } from "solid-js"

const [silenceMissingMappings, setSilenceMappingsMissing] = createSignal<boolean>(false)

export { silenceMissingMappings }

export default function Settings() {
  const [autoStartup, setAutoStartup] = createSignal<boolean>(false)
  isEnabled().then(setAutoStartup)

  const store = load("store.json")
  store.then(s => s.get<boolean>("silenceMissingMappings")).then(setSilenceMappingsMissing)

  async function toggleAutoStartup(_: any, checked: boolean) {
    await (checked ? enable : disable)()
    setAutoStartup(checked)
  }

  return <PageRoot>
    <div class="overflow-y-scroll m-4">
      <h1 class="font-bold text-3xl justify-self-start mb-2">Notifications</h1>
      <div class="border-2 border-indigo-200 rounded-xl p-2">
        <span>Silence "Mappings missing" notifications</span>
        <Switch onChange={async (_, checked) => {
          setSilenceMappingsMissing(checked)
          store.then(async s => {
            await s.set("silenceMissingMappings", checked)
            await s.save()
          })
        }} checked={silenceMissingMappings()} />
      </div>
      <br />
      <h1 class="font-bold text-3xl justify-self-start mb-2">Startup</h1>
      <div class="border-2 border-indigo-200 rounded-xl p-2">
        <span>Startup when computer powers on</span>
        <Switch onChange={toggleAutoStartup} checked={autoStartup()} />
      </div>
    </div>
  </PageRoot>
}

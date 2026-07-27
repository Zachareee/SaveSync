import Switch from "@suid/material/Switch"
import { enable, disable } from "@tauri-apps/plugin-autostart"

export default function Settings() {
  function toggleAutoStartup(_: any, checked: boolean) {
    (checked ? enable : disable)()
  }

  return <div class="overflow-y-scroll m-4">
    <h1 class="font-bold text-3xl justify-self-start mb-1">Startup</h1>
    <div class="border-2 border-indigo-200 rounded-xl p-2">
      <span>Startup when computer powers on</span>
      <Switch onChange={toggleAutoStartup} />
    </div>
  </div>
}

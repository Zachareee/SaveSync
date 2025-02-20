import { Menu, MenuItem, Submenu } from "@tauri-apps/api/menu";
import { createWindow } from "./window";

export const mainMenu = await Menu.new({
  items: await Promise.all([
    Submenu.new({
      text: "File", id: "file", items: [
        {
          text: "Mapping",
          id: "mapping",
          enabled: false,
          action() {
            createWindow("/mapping", { title: "Mapping", parent: "main" })
          }
        }
      ]
    })
  ])
})

export async function setWindowMenu() {
  return mainMenu.setAsWindowMenu()
}

export async function menuStatus(active: boolean) {
  return mainMenu.get("file").then(m => (m as Submenu).get("mapping")).then(i => (i as MenuItem).setEnabled(active))
}

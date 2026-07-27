import { folders, setFolders } from "@/App";
import DivButton from "@/components/DivButton";
import { emit, osStringToString, stringToOsString } from "@/logic/backend";
import { OsString } from "@/types/rust";
import { useNavigate, useParams } from "@solidjs/router";
import { Folder, InsertDriveFile, Loop } from "@suid/icons-material";
import { Index } from "solid-js";
import { Portal } from "solid-js/web";

export default function Folders() {
  const navigate = useNavigate()
  const params = useParams()
  const { TAGNAME } = Object.fromEntries(
    Object.entries(params).map(([k, v]) => [k, decodeURIComponent(v)])
  )

  const sync_folder = (foldername: OsString) => {
    setFolders(TAGNAME, osStringToString(foldername), "loading", true)
    emit("sync", { tag: TAGNAME, foldername })
  }

  return <>
    <div class="flex justify-center overflow-y-auto">
      <div class="w-min h-screen">
        <Index each={Object.entries(folders[TAGNAME])}>
          {
            foldername => <DivButton onclick={[sync_folder, stringToOsString(foldername()[0])]}>
              <input type="checkbox" class="mr-4 rounded-2xl" checked={foldername()[1].synced} onclick={(e) => e.preventDefault()} />
              {foldername()[1].folder ? <Folder /> : <InsertDriveFile />} {foldername()[0]}
              <Loop style={{ "visibility": foldername()[1].loading ? "visible" : "hidden" }} class="ml-2" />
            </DivButton>
          }
        </Index>
      </div>
      <Portal>
        <div class="fixed right-0 bottom-0 m-4">
          <button onclick={[navigate, "/tags"]}>Back to tags</button>
        </div>
      </Portal>
    </div>
  </>
}

import { folders, setFolders } from "@/logic/folder_state";
import DivButton from "@/components/DivButton";
import { emit, osStringToString, stringToOsString } from "@/logic/backend";
import PageRoot from "@/PageRoot";
import { OsString } from "@/types/rust";
import { useNavigate, useParams } from "@solidjs/router";
import Folder from "@suid/icons-material/Folder"
import InsertDriveFile from "@suid/icons-material/InsertDriveFile"
import Loop from "@suid/icons-material/Loop"
import { mapValues } from "lodash";
import { Index } from "solid-js";
import { Portal } from "solid-js/web";

export default function Folders() {
  const navigate = useNavigate()
  const { TAGNAME: tag } = mapValues(useParams(), i => decodeURIComponent(i ?? ""))

  const sync_folder = ({ foldername, is_folder }: { foldername: OsString, is_folder: boolean }) => {
    setFolders(tag, osStringToString(foldername), "loading", true)
    emit("sync", { tag, foldername, is_folder })
  }

  return <PageRoot>
    <div class="w-full h-screen flex-col content-center justify-center items-center">
      <Index each={Object.entries(folders[tag])}>
        {
          e => <DivButton onclick={[sync_folder, { foldername: stringToOsString(e()[0]), is_folder: e()[1].folder }]}>
            <input type="checkbox" class="mr-4 rounded-2xl" checked={e()[1].synced} onclick={(e) => e.preventDefault()} />
            {e()[1].folder ? <Folder /> : <InsertDriveFile />} {e()[0]}
            <Loop style={{ "visibility": e()[1].loading ? "visible" : "hidden" }} class="ml-2" />
          </DivButton>
        }
      </Index>
    </div>
    <Portal>
      <div class="fixed right-0 bottom-0 m-4">
        <button onclick={[navigate, "/tags"]}>Back to tags</button>
      </div>
    </Portal>
  </PageRoot>
}

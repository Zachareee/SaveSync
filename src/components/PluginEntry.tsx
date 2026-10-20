import { Info } from "@/types/data";
import { Component, JSX, Show } from "solid-js";
import Logout from "@suid/icons-material/Logout";

const PluginEntry: Component<JSX.HTMLAttributes<HTMLDivElement>
  & Record<"init" | "logout", () => void>
  & { elem: Info, loggedIn: boolean }>
  = ({ init, logout, elem: { icon_url, description, author, name }, loggedIn }) => {
    return <>
      <div onclick={init} class="flex border justify-end p-1.5 cursor-pointer rounded-lg">
        <Show when={icon_url}>
          <img src={icon_url} class="w-10 mr-4 inline-block content-center" />
        </Show>
        <div class="inline-block">
          <h2>{name}</h2>
          <p>Description: {description}</p>
          <span>Written by: {author}</span>
        </div>
        <div class="inline-block">
          <Show when={loggedIn}>
            <Logout class="bg-red-600 size-full content-center hover:outline-2" onClick={logout} />
          </Show>
        </div>
      </div>
    </>
  }

export default PluginEntry

export type AbortInfo = Pick<Info, "name" | "filename">

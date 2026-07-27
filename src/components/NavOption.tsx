import { Component, JSX } from "solid-js";

const NavOption: Component<JSX.HTMLAttributes<HTMLDivElement>> = props => {
  return <>
    <div onclick={props.onclick} class="text-center p-2 hover:bg-indigo-800 cursor-pointer">
      <h1 class="font-bold text-xl">{props.children}</h1>
    </div>
  </>
}

export default NavOption

import { Component, JSX } from "solid-js";

const DivButton: Component<JSX.HTMLAttributes<HTMLDivElement>> = props => {
  return <div onclick={props.onclick} class="w-full text-nowrap bg-slate-900 rounded-2xl p-4 m-2 select-none cursor-pointer hover:outline-2">
    {props.children}
  </div>
}

export default DivButton

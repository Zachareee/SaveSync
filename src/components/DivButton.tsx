import { JSX } from "solid-js";

export default function DivButton(props: { onclick: JSX.HTMLElementTags["div"]["onclick"], children: JSX.Element }) {
  return <div onclick={props.onclick} class="w-full text-nowrap bg-slate-900 rounded-2xl p-4 m-2 select-none">
    {props.children}
  </div>
}

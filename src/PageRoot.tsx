import { useLocation, useNavigate } from "@solidjs/router"
import { ParentComponent, Show } from "solid-js"
import NavBar from "./components/NavBar"

const navBarPaths = [/tags.*/, /mapping/, /settings/]

const PageRoot: ParentComponent = props => {
  const navigate = useNavigate()
  const location = useLocation().pathname

  return <div class="w-full flex">
    <Show when={navBarPaths.some(r => r.test(location))}>
      <NavBar navigate={navigate} />
    </Show>
    <div class="content-center flex-auto">
      {props.children}
    </div>
  </div>
}

export default PageRoot

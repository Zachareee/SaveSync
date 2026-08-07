import { useNavigate } from "@solidjs/router"
import { ParentComponent } from "solid-js"
import NavBar from "./components/NavBar"

const PageRoot: ParentComponent = props => {
  const navigate = useNavigate()

  return <div class="w-full flex">
    <NavBar navigate={navigate} />
    <div class="content-center flex-auto overflow-auto">
      {props.children}
    </div>
  </div>
}

export default PageRoot

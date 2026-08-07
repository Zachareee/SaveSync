import { useNavigate } from "@solidjs/router"
import { ParentComponent } from "solid-js"
import NavBar from "./components/NavBar"

const PageRoot: ParentComponent = props => {
  const navigate = useNavigate()

  return <div class="w-full flex">
    <NavBar navigate={navigate} />
    <div class="h-screen flex justify-center items-center flex-1 overflow-auto">
      {props.children}
    </div>
  </div>
}

export default PageRoot

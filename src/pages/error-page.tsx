import { useParams } from "@solidjs/router"

export default function ErrorPage() {
  const params = useParams()

  return <div>
    <p>Error found: {decodeURIComponent(params.ERROR)}</p>
  </div>
}

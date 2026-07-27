import { useSearchParams } from "@solidjs/router"

export default function ErrorPage() {
  const [params, _setter] = useSearchParams()

  return <div>
    <p>Error found: {params.error}</p>
  </div>
}

import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_admin/users')({
  component: RouteComponent,
})

function RouteComponent() {
  return <div>Hello "/_admin/users"!</div>
}

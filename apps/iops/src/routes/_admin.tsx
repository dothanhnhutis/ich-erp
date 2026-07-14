import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_admin")({
  component: RouteComponent,
  beforeLoad: ({ context, location }) => {
    if (!context.state?.data)
      throw redirect({ to: "/login", search: { redirect: location.href } });
  },
});

function RouteComponent() {
  return <div>Hello "/_admin"!</div>;
}

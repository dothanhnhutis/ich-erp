import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import z from "zod";

const fallback = "/users" as const;

export const Route = createFileRoute("/_auth")({
  validateSearch: z.object({
    redirect: z.string().optional().catch(""),
  }),
  component: RouteComponent,
  beforeLoad: ({ context, search }) => {
    if (context.auth.user) {
      throw redirect({ to: search.redirect || fallback });
    }
  },
});

function RouteComponent() {
  return <Outlet />;
}

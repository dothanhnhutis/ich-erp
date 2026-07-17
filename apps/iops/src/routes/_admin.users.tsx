import { api } from "@/lib/api";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/_admin/users")({
  component: RouteComponent,
});

function RouteComponent() {
  const query = useQuery({
    queryKey: ["users"],
    queryFn: api.listUsers,
  });

  console.log("query.data", query.data);

  return <div>Hello "/_admin/users"!</div>;
}

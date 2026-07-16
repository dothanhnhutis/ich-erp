import { AppSidebar } from "@/components/app-sidebar";
import NavHeader from "@/components/nav-header";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_admin")({
  component: RouteComponent,
  beforeLoad: ({ context, location }) => {
    if (!context.state?.data)
      throw redirect({ to: "/login", search: { redirect: location.href } });
  },
});

function RouteComponent() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <NavHeader />
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}

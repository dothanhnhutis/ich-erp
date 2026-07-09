import { ForgotPasswordForm } from "@/components/forgot-password-form";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/forgot-password")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    // <div className="flex min-h-svh flex-col items-center justify-center gap-6 bg-background p-6 md:p-10">
    //   <div className="w-full max-w-sm">
    //     <ForgotPasswordForm />
    //   </div>
    // </div>

    <div className="flex flex-col gap-4 p-6 md:p-10 h-screen">
      <div className="flex flex-1 items-center justify-center">
        <div className="w-full max-w-xs">
          <ForgotPasswordForm />
        </div>
      </div>
    </div>
  );
}

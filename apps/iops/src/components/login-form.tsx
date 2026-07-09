import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import ichLogo from "@/assets/logo.png";
import { Link } from "@tanstack/react-router";
import * as z from "zod";
import { useForm } from "@tanstack/react-form";
import { Spinner } from "./ui/spinner";
import { invoke } from "@tauri-apps/api/core";

const formSchema = z.object({
  email: z.email("Email và mật khẩu không hợp lệ."),
  password: z
    .string()
    .min(8, "Email và mật khẩu không hợp lệ.")
    .max(255, "Email và mật khẩu không hợp lệ."),
});

export function LoginForm({
  className,
  ...props
}: React.ComponentProps<"form">) {
  const form = useForm({
    defaultValues: {
      email: "",
      password: "",
    },
    validators: {
      onSubmit: formSchema,
    },
    onSubmit: async ({ value }) => {
      console.log(value);
      await invoke("login");
    },
  });
  return (
    <form
      className={cn("flex flex-col gap-6", className)}
      {...props}
      onSubmit={(e) => {
        e.preventDefault();
        form.handleSubmit();
      }}
    >
      <FieldGroup>
        <div className="flex flex-col items-center gap-1 text-center">
          <img src={ichLogo} alt="logo" className="mb-2 h-30 w-auto" />
        </div>
        <div className="flex flex-col items-center gap-1 text-center">
          <h1 className="text-2xl font-bold">Welcome to I.C.H IOPs</h1>
          <p className="text-sm text-muted-foreground text-balance">
            Nhập email và mật khẩu để đăng nhập vào I.C.H IOPs.
          </p>
        </div>
        <form.Field
          name="email"
          children={(field) => {
            const isInvalid =
              field.state.meta.isTouched && !field.state.meta.isValid;

            return (
              <Field>
                <FieldLabel htmlFor={field.name}>Email</FieldLabel>
                <Input
                  type="text"
                  id={field.name}
                  name={field.name}
                  aria-invalid={isInvalid}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="m@example.com"
                  required
                />
              </Field>
            );
          }}
        />

        <form.Field
          name="password"
          children={(field) => {
            const isInvalid =
              field.state.meta.isTouched && !field.state.meta.isValid;

            return (
              <Field>
                <div className="flex items-center">
                  <FieldLabel htmlFor={field.name}>Mật khẩu</FieldLabel>
                  <Link
                    tabIndex={-1}
                    to="/forgot-password"
                    className="ml-auto text-sm underline-offset-4 hover:underline"
                  >
                    Quên mật khẩu?
                  </Link>
                </div>
                <Input
                  type="password"
                  id={field.name}
                  name={field.name}
                  aria-invalid={isInvalid}
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(e) => field.handleChange(e.target.value)}
                  placeholder="*********"
                  required
                />
                {isInvalid && <FieldError errors={field.state.meta.errors} />}
              </Field>
            );
          }}
        />
        <Field>
          <Button type="submit">
            Đăng nhập
            <Spinner data-icon="inline-start" />
          </Button>
        </Field>
      </FieldGroup>
    </form>
  );
}

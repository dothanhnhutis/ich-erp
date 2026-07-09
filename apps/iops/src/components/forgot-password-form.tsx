import { GalleryVerticalEnd } from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
  FieldSeparator,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import ichLogo from "@/assets/logo.png";
import { Link } from "@tanstack/react-router";

export function ForgotPasswordForm({
  className,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div className={cn("flex flex-col gap-6", className)} {...props}>
      <form>
        <FieldGroup>
          <div className="flex flex-col items-center gap-1 text-center">
            <img src={ichLogo} alt="logo" className="mb-2 h-30 w-auto" />
          </div>
          <div className="flex flex-col items-center gap-1 text-center">
            <h1 className="text-2xl font-bold">Tìm lại bản thân</h1>
            <p className="text-sm text-muted-foreground">
              Nhập email và bấm <b>Gửi</b> để nhận được hướng dẫn lấy lại mật
              khẩu.
            </p>
          </div>
          <Field>
            <FieldLabel htmlFor="email">Email</FieldLabel>
            <Input
              id="email"
              type="email"
              placeholder="m@example.com"
              required
            />
          </Field>
          <Field>
            <Button type="submit">Gửi</Button>
          </Field>
        </FieldGroup>
      </form>
      <FieldDescription className="px-6 text-center">
        Đã tìm thấy bản thân? <Link to="/login">Đăng nhập</Link>.
      </FieldDescription>
    </div>
  );
}

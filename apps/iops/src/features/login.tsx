import { LoginForm } from "@/components/login-form";

const Login = () => {
  return (
    <div className="flex flex-col gap-4 p-6 md:p-10 h-screen">
      <div className="flex flex-1 items-center justify-center">
        <div className="w-full max-w-xs">
          <LoginForm />
        </div>
      </div>
    </div>
  );
};

export default Login;

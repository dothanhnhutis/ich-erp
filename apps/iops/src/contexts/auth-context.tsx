import { createContext, ReactNode, useContext, useMemo, useState } from "react";

type ProfileResponse = {};

export type AuthState = {
  profile: ProfileResponse | null;
  /** Đang kiểm tra session từ Rust State (chỉ true lúc khởi động). */
  hydrating: boolean;
};

type AuthContextValue = {
  auth: AuthState;
  //   login: (email: string, password: string) => Promise<void>;
  //   logout: () => Promise<void>;
  //   refreshProfile: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState>({
    profile: null,
    hydrating: true,
  });

  const value = useMemo(() => ({ auth }), [auth]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within <AuthProvider>");
  }
  return ctx;
}

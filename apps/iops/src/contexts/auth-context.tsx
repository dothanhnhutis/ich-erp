import { api, Session, userProfile } from "@/lib/api";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useMemo,
  useState,
} from "react";

export type AuthState = {
  session: Session | null;
  user: userProfile | null;
  permission_codes: string[];
  /** Đang kiểm tra session từ Rust State (chỉ true lúc khởi động). */
  hydrating: boolean;
};

type AuthContextValue = {
  auth: AuthState;
  login: (email: string, password: string) => Promise<void>;
  //   logout: () => Promise<void>;
  //   refreshProfile: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState>({
    session: null,
    user: null,
    permission_codes: [],
    hydrating: true,
  });

  const login = useCallback(async (email: string, password: string) => {
    const data = await api.login(email, password);
    setAuth({ ...data, hydrating: false });
  }, []);

  const value = useMemo(() => ({ auth, login }), [auth, login]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within <AuthProvider>");
  }
  return ctx;
}

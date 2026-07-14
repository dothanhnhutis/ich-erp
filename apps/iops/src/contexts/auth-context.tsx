import { api, LoginResponse } from "@/lib/api";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

export type AuthState = {
  data: LoginResponse | null;
  /** Đang kiểm tra session từ Rust State (chỉ true lúc khởi động). */
  hydrating: boolean;
};

export type AuthContextValue = {
  state: AuthState | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  //   refreshProfile: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({
    data: null,
    hydrating: true,
  });

  const login = useCallback(async (email: string, password: string) => {
    const data = await api.login(email, password);
    setState({ data, hydrating: false });
  }, []);

  const logout = useCallback(async () => {
    await api.logout();
    setState({ data: null, hydrating: false });
  }, []);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const data = await api.hydrate();
        if (cancelled) return;
        setState({ data, hydrating: false });
      } catch {
        if (cancelled) return;
        setState({
          data: null,
          hydrating: false,
        });
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const value = useMemo(
    () => ({ state, login, logout }),
    [state, login, logout],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth must be used within <AuthProvider>");
  }
  return ctx;
}

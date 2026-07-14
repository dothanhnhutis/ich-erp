import { invoke } from "@tauri-apps/api/core";

export class ApiError extends Error {
  constructor(
    public readonly kind: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

type RustErrorShape = {
  kind: string;
  message?: string;
  status?: number;
};

async function call<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    if (typeof e === "object" && e !== null && "kind" in e) {
      const err = e as RustErrorShape;
      throw new ApiError(err.kind, err.message ?? err.kind);
    }
    throw new ApiError(
      "Unknown",
      typeof e === "string" ? e : JSON.stringify(e),
    );
  }
}
export type UserStatus = "ACTIVE" | "DEACTIVATED" | "PENDING_PASSWORD";

export type ProfileResponse = {
  id: string;
  email: string;
  username: string | null;
  status: UserStatus;
  created_at: string;
  updated_at: string;
  permissions: string[];
};

export type Session = {
  id: string;
  user_id: string;
  token_hash: string;
  device_id: string | null;
  device_name: string | null;
  device_type: string;
  platform: string | null;
  app_version: string | null;
  user_agent: string | null;
  ip_address: string | null;
  revoked_at: string | null;
  revoke_reason: string | null;
  expires_at: string;
  created_at: string;
  updated_at: string;
};

export type userProfile = {
  id: string;
  email: string;
  username: string | null;
  status: UserStatus;
  deactivated_at: string | null;
  created_at: string;
  updated_at: string;
};

export type LoginResponse = {
  session: Session;
  user: userProfile;
  permission_codes: string[];
};

export const api = {
  // Auth — KHÔNG dùng authedCall
  login: (email: string, password: string) =>
    call<LoginResponse>("login", {
      payload: { email, password, device_type: "web" },
    }),
  hydrate: () => call<LoginResponse | null>("hydrate"),
  logout: () => call<void>("logout"),
};

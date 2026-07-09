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

export type LoginOutcome = {
  user_id: string;
  profile: ProfileResponse;
};

export const api = {
  // Auth — KHÔNG dùng authedCall
  login: (email: string, password: string) =>
    call<LoginOutcome>("api_login", { email, password }),
};

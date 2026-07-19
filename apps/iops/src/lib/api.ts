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

/**
 * Gọi command authed: nếu gặp 401 thì tự refresh token và retry 1 lần.
 * Dùng cho mọi endpoint cần access token. KHÔNG dùng cho login/logout/refresh/hydrate.
 */
async function authedCall<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await call<T>(cmd, args);
  } catch (e) {
    if (e instanceof ApiError && e.kind === "Unauthorized") {
      // await call<null>("api_refresh");
      return await call<T>(cmd, args);
    }
    throw e;
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

export type ListParams = {
  page?: number;
  pageSize?: number;
  q?: string;
};

export type PaginatedResponse<T> = {
  items: T[];
  total: number;
  page: number;
  page_size: number;
};

export type UserResponse = {
  id: string;
  email: string;
  username: string | null;
  status: UserStatus;
  deactivated_at: string | null;
  // roles: RoleResponse[];
  created_at: string;
  updated_at: string;
};

export type PermissionResponse = {
  id: string;
  code: string;
  description: string;
};
export type RoleResponse = {
  id: string;
  name: string;
  description: string;
  status: string;
  can_delete: boolean;
  can_update: boolean;
  permissions: PermissionResponse[];
  created_at: string;
  updated_at: string;
};

export const api = {
  // Auth — KHÔNG dùng authedCall
  login: (email: string, password: string) =>
    call<LoginResponse>("login", {
      payload: { email, password, device_type: "web" },
    }),
  hydrate: () => call<LoginResponse | null>("hydrate"),
  logout: () => call<void>("logout"),

  // User
  listUsers: (params: ListParams = {}) =>
    call<PaginatedResponse<UserResponse>>("list_users", {
      page: params.page,
      pageSize: params.pageSize,
      q: params.q,
    }),

  listRoles: (params: ListParams = {}) =>
    call<PaginatedResponse<RoleResponse>>("list_roles", {
      page: params.page,
      pageSize: params.pageSize,
      q: params.q,
    }),

  me: () => call<UserResponse>("me"),
};

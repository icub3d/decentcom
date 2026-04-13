import { useCallback, useState } from "react";
import { authenticateWithServer, AuthFlowInput, VerifyResponse } from "../api/auth";

export type AuthStatus = "unauthenticated" | "authenticating" | "authenticated";

export interface AuthState {
  status: AuthStatus;
  token: string | null;
  userId: string | null;
  expiresAt: string | null;
  error: string | null;
}

const initialState: AuthState = {
  status: "unauthenticated",
  token: null,
  userId: null,
  expiresAt: null,
  error: null,
};

export function useAuth() {
  const [state, setState] = useState<AuthState>(initialState);

  const authenticate = useCallback(async (input: AuthFlowInput): Promise<VerifyResponse> => {
    setState((prev) => ({ ...prev, status: "authenticating", error: null }));
    try {
      const result = await authenticateWithServer(input);
      setState({
        status: "authenticated",
        token: result.token,
        userId: result.user_id,
        expiresAt: result.expires_at,
        error: null,
      });
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setState((prev) => ({
        ...prev,
        status: "unauthenticated",
        token: null,
        userId: null,
        expiresAt: null,
        error: message,
      }));
      throw err;
    }
  }, []);

  const clearAuth = useCallback(() => {
    setState(initialState);
  }, []);

  return {
    ...state,
    authenticate,
    clearAuth,
    isAuthenticated: state.status === "authenticated",
  };
}

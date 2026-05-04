// @specre 01KQS7YE9H3RWJSHBNWPEJGYAV
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
} from "react";
import type { ReactNode } from "react";

const TOKEN_KEY = "cms_auth_token";

interface AuthContextValue {
	token: string | null;
	signIn: (token: string) => void;
	signOut: () => void;
	isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
	const [token, setToken] = useState<string | null>(() =>
		localStorage.getItem(TOKEN_KEY),
	);

	const signIn = useCallback((newToken: string) => {
		localStorage.setItem(TOKEN_KEY, newToken);
		setToken(newToken);
	}, []);

	const signOut = useCallback(() => {
		localStorage.removeItem(TOKEN_KEY);
		setToken(null);
	}, []);

	useEffect(() => {
		const stored = localStorage.getItem(TOKEN_KEY);
		if (stored !== token) setToken(stored);
	}, [token]);

	const value = useMemo(
		() => ({ token, signIn, signOut, isAuthenticated: !!token }),
		[token, signIn, signOut],
	);

	return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthContextValue {
	const ctx = useContext(AuthContext);
	if (!ctx) throw new Error("useAuth must be used within AuthProvider");
	return ctx;
}

export function getStoredToken(): string | null {
	return localStorage.getItem(TOKEN_KEY);
}

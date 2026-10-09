import { useEffect, useState } from "react";

const TOKEN_KEY = "botcast_token";

export const getToken = (): string | null => localStorage.getItem(TOKEN_KEY);
export const setToken = (token: string) =>
	localStorage.setItem(TOKEN_KEY, token);
export const clearToken = () => localStorage.removeItem(TOKEN_KEY);

interface JwtClaims {
	sub: string;
	exp: number;
}

const decodeToken = (token: string): JwtClaims | null => {
	try {
		const payload = token.split(".")[1] ?? "";
		const json = atob(payload.replace(/-/g, "+").replace(/_/g, "/"));
		return JSON.parse(json) as JwtClaims;
	} catch {
		return null;
	}
};

const isExpired = (claims: JwtClaims) => claims.exp * 1000 <= Date.now();

/** 有効なトークンの subject (botcast-cms のユーザー ID, 例: `user:xxx`) を返す */
export const getUserId = (): string | null => {
	const token = getToken();
	if (!token) return null;
	const claims = decodeToken(token);
	if (!claims || isExpired(claims)) return null;
	return claims.sub;
};

const readValidToken = (): string | null => {
	const token = getToken();
	if (!token) return null;
	const claims = decodeToken(token);
	if (!claims || isExpired(claims)) {
		clearToken();
		return null;
	}
	return token;
};

export const useSession = () => {
	const [token, setTokenState] = useState<string | null>(() =>
		readValidToken(),
	);

	useEffect(() => {
		const handler = () => setTokenState(readValidToken());
		window.addEventListener("storage", handler);
		return () => window.removeEventListener("storage", handler);
	}, []);

	const saveToken = (t: string) => {
		setToken(t);
		setTokenState(t);
	};

	const removeToken = () => {
		clearToken();
		setTokenState(null);
	};

	const claims = token ? decodeToken(token) : null;

	return {
		token,
		userId: claims?.sub ?? null,
		saveToken,
		removeToken,
		isSignedIn: token !== null,
	};
};

import { useEffect, useState } from "react";

const TOKEN_KEY = "botcast_token";

export const getToken = (): string | null => localStorage.getItem(TOKEN_KEY);
export const setToken = (token: string) =>
	localStorage.setItem(TOKEN_KEY, token);
export const clearToken = () => localStorage.removeItem(TOKEN_KEY);

export const useSession = () => {
	const [token, setTokenState] = useState<string | null>(() => getToken());

	useEffect(() => {
		const handler = () => setTokenState(getToken());
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

	return {
		token,
		saveToken,
		removeToken,
		isSignedIn: token !== null,
	};
};

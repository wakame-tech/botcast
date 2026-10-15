import { initializeApp } from "firebase/app";
import {
	GoogleAuthProvider,
	getAuth,
	inMemoryPersistence,
	setPersistence,
	signInWithPopup,
	signOut,
} from "firebase/auth";
import { CMS_TENANT, CMS_URL } from "./cms_client";

// Firebase の Web config は公開値。別プロジェクトを使うときは VITE_FIREBASE_* で上書きする
const app = initializeApp({
	apiKey:
		import.meta.env.VITE_FIREBASE_API_KEY ??
		"AIzaSyBWgxz2n9U1akZYh7d8-2UmniFnYGVDsW8",
	authDomain:
		import.meta.env.VITE_FIREBASE_AUTH_DOMAIN ?? "botcast-cms.firebaseapp.com",
	projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID ?? "botcast-cms",
});

/** Google でログインし、Firebase の ID トークンを botcast-cms の JWT に交換する */
export async function signInWithGoogle(): Promise<string> {
	const auth = getAuth(app);
	// 以後は CMS の JWT だけを使うので Firebase のセッションは残さない
	await setPersistence(auth, inMemoryPersistence);
	const credential = await signInWithPopup(auth, new GoogleAuthProvider());
	const idToken = await credential.user.getIdToken();
	await signOut(auth);

	const res = await fetch(`${CMS_URL}/t/${CMS_TENANT}/auth/firebase`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ id_token: idToken }),
	});
	if (!res.ok) {
		const body = await res.json().catch(() => ({}));
		throw new Error(body.message ?? "サインインに失敗しました");
	}
	const { token } = await res.json();
	return token;
}

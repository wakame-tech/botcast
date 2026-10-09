import { Button } from "@/components/ui/button";
import { useSession } from "@/hooks/useSession";
import { CMS_URL } from "@/lib/cms_client";
import { useNavigate } from "@tanstack/react-router";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/signin")({
	component: Signin,
});

function Signin() {
	const { saveToken } = useSession();
	const navigate = useNavigate();
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);
	const [mode, setMode] = useState<"signin" | "signup">("signin");
	const label = mode === "signin" ? "サインイン" : "サインアップ";

	const handleSignIn = async (e: React.FormEvent) => {
		e.preventDefault();
		setLoading(true);
		setError(null);
		try {
			const res = await fetch(`${CMS_URL}/auth/${mode}`, {
				method: "POST",
				headers: { "Content-Type": "application/json" },
				body: JSON.stringify({ email, password }),
			});
			if (!res.ok) {
				const body = await res.json().catch(() => ({}));
				setError(body.message ?? `${label}に失敗しました`);
				return;
			}
			const { token } = await res.json();
			saveToken(token);
			navigate({ to: "/" });
		} catch {
			setError("ネットワークエラーが発生しました");
		} finally {
			setLoading(false);
		}
	};

	return (
		<form
			onSubmit={handleSignIn}
			className="flex flex-col gap-4 max-w-sm mx-auto mt-16 p-4"
		>
			<h2 className="text-xl font-bold">{label}</h2>
			{error && <p className="text-red-500 text-sm">{error}</p>}
			<input
				type="email"
				placeholder="メールアドレス"
				value={email}
				onChange={(e) => setEmail(e.target.value)}
				required
				className="border rounded px-3 py-2"
			/>
			<input
				type="password"
				placeholder="パスワード"
				value={password}
				onChange={(e) => setPassword(e.target.value)}
				required
				className="border rounded px-3 py-2"
			/>
			<Button type="submit" disabled={loading}>
				{loading ? `${label}中...` : label}
			</Button>
			<button
				type="button"
				className="text-sm text-teal-700 underline bg-transparent border-none cursor-pointer"
				onClick={() => {
					setMode(mode === "signin" ? "signup" : "signin");
					setError(null);
				}}
			>
				{mode === "signin"
					? "アカウントをお持ちでない方はサインアップ"
					: "アカウントをお持ちの方はサインイン"}
			</button>
		</form>
	);
}

import { Button } from "@/components/ui/button";
import { useSession } from "@/hooks/useSession";
import { signInWithGoogle } from "@/lib/firebase";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/signin")({
	component: Signin,
});

function Signin() {
	const { saveToken } = useSession();
	const navigate = useNavigate();
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const handleSignIn = async () => {
		setLoading(true);
		setError(null);
		try {
			saveToken(await signInWithGoogle());
			navigate({ to: "/" });
		} catch (e) {
			setError(e instanceof Error ? e.message : "サインインに失敗しました");
		} finally {
			setLoading(false);
		}
	};

	return (
		<div className="flex flex-col gap-4 max-w-sm mx-auto mt-16 p-4">
			<h2 className="text-xl font-bold">サインイン</h2>
			{error && <p className="text-red-500 text-sm">{error}</p>}
			<Button onClick={handleSignIn} disabled={loading}>
				{loading ? "サインイン中..." : "Google でサインイン"}
			</Button>
		</div>
	);
}

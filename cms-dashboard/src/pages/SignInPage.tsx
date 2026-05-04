// @specre 01KQS7YE9H3RWJSHBNWPEJGYAV
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../contexts/AuthContext";

const BASE_URL = "";

async function callAuthApi(
	path: string,
	email: string,
	password: string,
): Promise<string> {
	const res = await fetch(`${BASE_URL}${path}`, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({ email, password }),
	});
	if (!res.ok) {
		const text = await res.text();
		throw new Error(text || `${res.status} ${res.statusText}`);
	}
	const data = (await res.json()) as { token: string };
	return data.token;
}

export function SignInPage() {
	const { signIn } = useAuth();
	const navigate = useNavigate();
	const [email, setEmail] = useState("");
	const [password, setPassword] = useState("");
	const [mode, setMode] = useState<"signin" | "signup">("signin");
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	async function handleSubmit(e: React.FormEvent) {
		e.preventDefault();
		setError(null);
		setLoading(true);
		try {
			const path = mode === "signin" ? "/auth/signin" : "/auth/signup";
			const token = await callAuthApi(path, email, password);
			signIn(token);
			navigate("/");
		} catch (err) {
			setError(err instanceof Error ? err.message : "Authentication failed");
		} finally {
			setLoading(false);
		}
	}

	return (
		<div
			style={{
				display: "flex",
				justifyContent: "center",
				alignItems: "center",
				height: "100vh",
				background: "#f5f5f5",
			}}
		>
			<div
				style={{
					background: "#fff",
					padding: "2rem",
					borderRadius: "8px",
					boxShadow: "0 2px 8px rgba(0,0,0,0.12)",
					width: "360px",
				}}
			>
				<h2 style={{ marginBottom: "1.5rem" }}>
					{mode === "signin" ? "Sign In" : "Sign Up"} — Botcast CMS
				</h2>
				<form onSubmit={handleSubmit}>
					<div style={{ marginBottom: "1rem" }}>
						<label style={{ display: "block", marginBottom: "4px" }}>
							Email
						</label>
						<input
							type="email"
							value={email}
							onChange={(e) => setEmail(e.target.value)}
							required
							style={{ width: "100%", padding: "8px", boxSizing: "border-box" }}
						/>
					</div>
					<div style={{ marginBottom: "1rem" }}>
						<label style={{ display: "block", marginBottom: "4px" }}>
							Password
						</label>
						<input
							type="password"
							value={password}
							onChange={(e) => setPassword(e.target.value)}
							required
							style={{ width: "100%", padding: "8px", boxSizing: "border-box" }}
						/>
					</div>
					{error && (
						<p style={{ color: "red", marginBottom: "1rem" }}>{error}</p>
					)}
					<button
						type="submit"
						disabled={loading}
						style={{
							width: "100%",
							padding: "10px",
							background: "#3b82f6",
							color: "#fff",
							border: "none",
							borderRadius: "4px",
							cursor: loading ? "not-allowed" : "pointer",
						}}
					>
						{loading ? "..." : mode === "signin" ? "Sign In" : "Sign Up"}
					</button>
				</form>
				<p style={{ marginTop: "1rem", textAlign: "center", fontSize: "14px" }}>
					{mode === "signin" ? (
						<>
							No account?{" "}
							<button
								style={{
									background: "none",
									border: "none",
									color: "#3b82f6",
									cursor: "pointer",
								}}
								onClick={() => setMode("signup")}
							>
								Sign Up
							</button>
						</>
					) : (
						<>
							Have an account?{" "}
							<button
								style={{
									background: "none",
									border: "none",
									color: "#3b82f6",
									cursor: "pointer",
								}}
								onClick={() => setMode("signin")}
							>
								Sign In
							</button>
						</>
					)}
				</p>
			</div>
		</div>
	);
}

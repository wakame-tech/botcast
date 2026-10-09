import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { getToken } from "@/hooks/useSession";
import { CMS_URL } from "@/lib/cms_client";
import { useState } from "react";

interface ChangePasswordFormProps {
	onChanged: (token: string) => void;
}

export function ChangePasswordForm(props: ChangePasswordFormProps) {
	const [currentPassword, setCurrentPassword] = useState("");
	const [newPassword, setNewPassword] = useState("");
	const [confirmPassword, setConfirmPassword] = useState("");
	const [error, setError] = useState<string | null>(null);
	const [message, setMessage] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setError(null);
		setMessage(null);
		if (newPassword.length === 0) {
			setError("新しいパスワードを入力してください");
			return;
		}
		if (newPassword !== confirmPassword) {
			setError("新しいパスワードと確認用パスワードが一致しません");
			return;
		}
		setLoading(true);
		try {
			const res = await fetch(`${CMS_URL}/auth/change-password`, {
				method: "POST",
				headers: {
					"Content-Type": "application/json",
					Authorization: `Bearer ${getToken() ?? ""}`,
				},
				body: JSON.stringify({
					current_password: currentPassword,
					new_password: newPassword,
				}),
			});
			if (!res.ok) {
				// 現在のパスワードが違う場合も 401 が返る
				if (res.status === 401) {
					setError("現在のパスワードが正しくありません");
				} else {
					const text = await res.text().catch(() => "");
					setError(text || "パスワードの変更に失敗しました");
				}
				return;
			}
			const { token } = await res.json();
			props.onChanged(token);
			setCurrentPassword("");
			setNewPassword("");
			setConfirmPassword("");
			setMessage("パスワードを変更しました");
		} catch {
			setError("ネットワークエラーが発生しました");
		} finally {
			setLoading(false);
		}
	};

	return (
		<form onSubmit={handleSubmit} className="flex flex-col gap-4 max-w-sm">
			<h3 className="text-lg font-bold">パスワード変更</h3>
			{error && <p className="text-red-500 text-sm">{error}</p>}
			{message && <p className="text-teal-700 text-sm">{message}</p>}
			<div className="flex flex-col gap-2">
				<Label htmlFor="current-password">現在のパスワード</Label>
				<Input
					id="current-password"
					type="password"
					autoComplete="current-password"
					value={currentPassword}
					onChange={(e) => setCurrentPassword(e.target.value)}
					required
				/>
			</div>
			<div className="flex flex-col gap-2">
				<Label htmlFor="new-password">新しいパスワード</Label>
				<Input
					id="new-password"
					type="password"
					autoComplete="new-password"
					value={newPassword}
					onChange={(e) => setNewPassword(e.target.value)}
					required
				/>
			</div>
			<div className="flex flex-col gap-2">
				<Label htmlFor="confirm-password">新しいパスワード（確認）</Label>
				<Input
					id="confirm-password"
					type="password"
					autoComplete="new-password"
					value={confirmPassword}
					onChange={(e) => setConfirmPassword(e.target.value)}
					required
				/>
			</div>
			<Button type="submit" disabled={loading}>
				{loading ? "変更中..." : "パスワードを変更"}
			</Button>
		</form>
	);
}

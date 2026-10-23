import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { $cms, CMS_TENANT } from "@/lib/cms_client";
import { useState } from "react";

/** CMS の秘密情報の名前（worker が台本生成で取り出すもの） */
const SECRET_NAME = "anthropic_api_key";

/**
 * 本人の LLM の API キー (BYOK)。CMS の `/me/secrets` に暗号化して保存する。
 * web から読めるのは末尾 4 文字と日時だけで、値は worker が台本生成のジョブで代理して取り出す
 */
export function LlmApiKeyForm() {
	const { data: secrets, refetch } = $cms.useQuery(
		"get",
		"/t/{tenantId}/me/secrets",
		{ params: { path: { tenantId: CMS_TENANT } } },
	);
	const putSecret = $cms.useMutation("put", "/t/{tenantId}/me/secrets/{name}");
	const deleteSecret = $cms.useMutation(
		"delete",
		"/t/{tenantId}/me/secrets/{name}",
	);

	const secret = secrets?.find((s) => s.name === SECRET_NAME);
	const [value, setValue] = useState("");
	const [message, setMessage] = useState<string | null>(null);
	const pending = putSecret.isPending || deleteSecret.isPending;
	const path = { tenantId: CMS_TENANT, name: SECRET_NAME };

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		const trimmed = value.trim();
		if (!trimmed) return;
		setMessage(null);
		const ok = await putSecret
			.mutateAsync({ params: { path }, body: { value: trimmed } })
			.then(() => true)
			.catch(() => false);
		// 失敗しても入力した値は画面に残さない
		setValue("");
		setMessage(
			ok
				? "保存しました"
				: "保存に失敗しました（Anthropic のキーは sk-ant- で始まります）",
		);
		await refetch();
	};

	const handleDelete = async () => {
		if (!confirm("LLM の API キーを削除しますか？台本の生成ができなくなります"))
			return;
		setMessage(null);
		const ok = await deleteSecret
			.mutateAsync({ params: { path } })
			.then(() => true)
			.catch(() => false);
		setMessage(ok ? "削除しました" : "削除に失敗しました");
		await refetch();
	};

	return (
		<form onSubmit={handleSubmit} className="flex flex-col gap-2 text-sm">
			<span className="font-bold">LLM の API キー</span>
			<span className="text-gray-500">
				台本の生成に使う Anthropic の API
				キー。費用はこのキーの持ち主に請求されます
			</span>
			{secret ? (
				<span>
					設定済み（末尾 <code>{secret.last4}</code>・更新{" "}
					{new Date(secret.updated_at).toLocaleString()}
					{secret.last_used_at &&
						`・最終使用 ${new Date(secret.last_used_at).toLocaleString()}`}
					）
				</span>
			) : (
				<span className="text-yellow-700">
					未設定です。設定するまで台本の生成は失敗します
				</span>
			)}
			<Input
				type="password"
				autoComplete="off"
				placeholder={secret ? "新しいキーで置き換える" : "sk-ant-..."}
				value={value}
				onChange={(e) => setValue(e.target.value)}
			/>
			<div className="flex items-center gap-2">
				<Button type="submit" disabled={pending || !value.trim()}>
					{secret ? "置き換える" : "保存"}
				</Button>
				{secret && (
					<Button
						type="button"
						variant="destructive"
						disabled={pending}
						onClick={handleDelete}
					>
						削除
					</Button>
				)}
				{message && <span className="text-gray-500">{message}</span>}
			</div>
		</form>
	);
}

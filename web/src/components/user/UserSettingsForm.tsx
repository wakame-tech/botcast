import { Button } from "@/components/ui/button";
import {
	$cms,
	CMS_TENANT,
	recordToLlmModel,
	recordToUserSettings,
	toRecordData,
} from "@/lib/cms_client";
import { useEffect, useState } from "react";

/** 「既定のモデルを使う」の選択肢の値 */
const DEFAULT_MODEL = "";

/**
 * 本人のユーザー設定 (CMS の `user_settings`)。今は LLM のモデルだけ。
 * API キー (BYOK) は秘密情報なので user_settings には置かず、ここに CMS の `/me/secrets` の欄を足す
 */
export function UserSettingsForm() {
	const { data: modelRecords } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "llm_models" } },
		},
	);
	// user_settings は read: owner なので、自分の設定だけが返る
	const { data: settingsRecords, refetch } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: {
				path: { tenantId: CMS_TENANT, collectionId: "user_settings" },
				query: { sort: "created_at", order: "desc", limit: 1 },
			},
		},
	);
	const createSettings = $cms.useMutation(
		"post",
		"/t/{tenantId}/records/{collectionId}",
	);
	const updateSettings = $cms.useMutation(
		"put",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	const models = (modelRecords ?? [])
		.map(recordToLlmModel)
		.sort((a, b) => a.order - b.order);
	const settingsRecord = settingsRecords?.[0];
	const settings = settingsRecord ? recordToUserSettings(settingsRecord) : null;
	const defaultModel = models.find((m) => m.default && m.enabled);
	const savedModel = settings?.llm_model ?? DEFAULT_MODEL;
	// 無効になった・消えたモデルが設定に残っていたら、worker と同じく既定を使う
	const savedIsUsable =
		savedModel === DEFAULT_MODEL ||
		models.some((m) => m.id === savedModel && m.enabled);

	const [llmModel, setLlmModel] = useState(DEFAULT_MODEL);
	const [message, setMessage] = useState<string | null>(null);
	useEffect(() => {
		setLlmModel(savedIsUsable ? savedModel : DEFAULT_MODEL);
	}, [savedModel, savedIsUsable]);

	const pending = createSettings.isPending || updateSettings.isPending;

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setMessage(null);
		// data は丸ごと置き換わるため、既存の値を保って更新する
		const { llm_model: _, ...current } = (settingsRecord?.data ?? {}) as Record<
			string,
			unknown
		>;
		const data = toRecordData(
			llmModel === DEFAULT_MODEL
				? current
				: { ...current, llm_model: llmModel },
		);
		const saved = settingsRecord
			? updateSettings.mutateAsync({
					params: {
						path: {
							tenantId: CMS_TENANT,
							collectionId: "user_settings",
							recordId: settingsRecord.id,
						},
					},
					body: { data },
				})
			: createSettings.mutateAsync({
					params: {
						path: { tenantId: CMS_TENANT, collectionId: "user_settings" },
					},
					body: { data },
				});
		const ok = await saved.then(() => true).catch(() => false);
		setMessage(ok ? "保存しました" : "保存に失敗しました");
		await refetch();
	};

	return (
		<form onSubmit={handleSubmit} className="flex flex-col gap-4">
			<label className="flex flex-col gap-1 text-sm">
				<span className="font-bold">LLM のモデル</span>
				<span className="text-gray-500">台本の生成に使うモデル</span>
				<select
					className="border rounded px-2 py-1"
					value={llmModel}
					onChange={(e) => setLlmModel(e.target.value)}
				>
					<option value={DEFAULT_MODEL}>
						既定{defaultModel ? ` (${defaultModel.name})` : ""}
					</option>
					{models
						.filter((m) => m.enabled)
						.map((m) => (
							<option key={m.id} value={m.id}>
								{m.name}
							</option>
						))}
				</select>
				{!savedIsUsable && (
					<span className="text-yellow-700">
						設定していたモデルは使えなくなったため、既定のモデルを使います
					</span>
				)}
			</label>
			<div className="flex items-center gap-2">
				<Button type="submit" disabled={pending}>
					保存
				</Button>
				{message && <span className="text-sm text-gray-500">{message}</span>}
			</div>
		</form>
	);
}

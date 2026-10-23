import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
	$cms,
	CMS_TENANT,
	type PodcastSchedule,
	type ScheduleGenerator,
	recordToFetchAllowedHost,
	recordToScript,
} from "@/lib/cms_client";
import { useState } from "react";

interface ScheduleFormProps {
	value?: PodcastSchedule;
	onSubmit: (schedule: PodcastSchedule) => void;
}

/** 事前に取得する資料の URL の数の上限 (worker の `MAX_SOURCES` と同じ) */
const MAX_SOURCES = 10;

/** テキストエリアの URL (1 行に 1 つ) を配列にする */
const parseSources = (text: string): string[] =>
	text
		.split("\n")
		.map((s) => s.trim())
		.filter((s) => s.length > 0);

/** URL のホストが取得を許可されたホストか (worker も同じ確認をする。ここでは入力の目安) */
const sourceProblem = (url: string, hosts: string[]): string | null => {
	let parsed: URL;
	try {
		parsed = new URL(url);
	} catch {
		return "URL の形式ではありません";
	}
	if (parsed.protocol !== "https:") return "https の URL だけ使えます";
	if (!hosts.includes(parsed.hostname.toLowerCase()))
		return `${parsed.hostname} は取得を許可されたホストではありません`;
	return null;
};

/** 番組の定期実行 (エピソードの自動生成) の設定 */
export function ScheduleForm({ value, onSubmit }: ScheduleFormProps) {
	const [cron, setCron] = useState(value?.cron ?? "0 0 12 * * Fri *");
	const [generator, setGenerator] = useState<ScheduleGenerator>(
		value?.generator ?? "script",
	);
	const [scriptId, setScriptId] = useState(value?.script_id ?? "");
	const [enabled, setEnabled] = useState(value?.enabled ?? false);
	const [args, setArgs] = useState<Record<string, unknown>>(
		value?.arguments ?? {},
	);
	const [prompt, setPrompt] = useState(value?.prompt ?? "");
	const [sourcesText, setSourcesText] = useState(
		(value?.sources ?? []).join("\n"),
	);
	const { data: records } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "scripts" } },
		},
	);
	const { data: hostRecords } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: {
				path: { tenantId: CMS_TENANT, collectionId: "fetch_allowed_hosts" },
			},
		},
	);
	const scripts = (records ?? []).map(recordToScript);
	const schema = scripts.find((s) => s.id === scriptId)?.arguments ?? {};
	const allowedHosts = (hostRecords ?? [])
		.map(recordToFetchAllowedHost)
		.filter((h) => h.enabled);
	const sources = parseSources(sourcesText);
	const problems = sources
		.map((url) => ({
			url,
			problem: sourceProblem(
				url,
				allowedHosts.map((h) => h.host),
			),
		}))
		.filter((p) => p.problem !== null);
	const tooMany = sources.length > MAX_SOURCES;

	return (
		<form
			className="flex flex-col gap-2 py-4"
			onSubmit={(e) => {
				e.preventDefault();
				if (tooMany) return;
				const schedule: PodcastSchedule = { cron, generator, enabled };
				if (generator === "script") {
					schedule.script_id = scriptId;
					schedule.arguments = args;
				} else {
					schedule.prompt = prompt;
					// LLM モードでもスクリプトの選択は残しておける (切り替えたときに戻せるように)
					if (scriptId) schedule.script_id = scriptId;
				}
				if (sources.length > 0) schedule.sources = sources;
				onSubmit(schedule);
			}}
		>
			<h2 className="text-lg font-bold">定期実行</h2>
			<label className="text-sm" htmlFor="schedule-cron">
				cron (秒 分 時 日 月 曜日 年、UTC。例: 毎週月曜 7:00 JST は「0 0 22 * *
				Sun *」)
			</label>
			<Input
				id="schedule-cron"
				value={cron}
				onChange={(e) => setCron(e.target.value)}
			/>
			<label className="text-sm">
				台本の作り方
				<select
					className="block border rounded px-2 py-1"
					value={generator}
					onChange={(e) => setGenerator(e.target.value as ScheduleGenerator)}
				>
					<option value="script">スクリプト</option>
					<option value="llm">LLM (マイページの API キー・モデルを使う)</option>
				</select>
			</label>
			{generator === "script" ? (
				<>
					<label className="text-sm">
						スクリプト
						<select
							className="block border rounded px-2 py-1"
							value={scriptId}
							onChange={(e) => {
								// スクリプトが変わると引数のスキーマも変わるので値を捨てる
								setScriptId(e.target.value);
								setArgs({});
							}}
							required={enabled}
						>
							<option value="">選択してください</option>
							{scripts.map((s) => (
								<option key={s.id} value={s.id}>
									{s.title}
								</option>
							))}
						</select>
					</label>
					{Object.keys(schema).length !== 0 && (
						<div className="text-sm">
							引数
							<JsonSchemaForm
								key={scriptId}
								schema={schema}
								formData={args}
								onChange={setArgs}
								hideSubmit
							/>
						</div>
					)}
				</>
			) : (
				<>
					<label className="text-sm" htmlFor="schedule-prompt">
						LLM への指示 (番組の配役は自動で渡します。資料の URL は fetch_url
						ツールで開けます)
					</label>
					<textarea
						id="schedule-prompt"
						className="border rounded px-2 py-1 min-h-40 text-sm"
						value={prompt}
						onChange={(e) => setPrompt(e.target.value)}
						maxLength={8000}
						required
					/>
				</>
			)}
			<label className="text-sm" htmlFor="schedule-sources">
				事前に取得する資料の URL (1 行に 1 つ、{MAX_SOURCES} 個まで。
				{generator === "script"
					? "スクリプトに context.sources として渡します"
					: "LLM に渡します"}
				)
			</label>
			<textarea
				id="schedule-sources"
				className="border rounded px-2 py-1 min-h-16 text-sm font-mono"
				value={sourcesText}
				onChange={(e) => setSourcesText(e.target.value)}
				placeholder="https://zenn.dev/api/articles?order=weekly&count=5"
			/>
			{tooMany && (
				<p className="text-sm text-red-600">URL は {MAX_SOURCES} 個までです</p>
			)}
			{problems.map((p) => (
				<p key={p.url} className="text-sm text-red-600">
					{p.url}: {p.problem}
				</p>
			))}
			<p className="text-sm text-gray-500">
				取得を許可されたホスト (管理者が設定):{" "}
				{allowedHosts.length === 0
					? "なし"
					: allowedHosts.map((h) => h.host).join(", ")}
			</p>
			<label className="text-sm flex items-center gap-2">
				<input
					type="checkbox"
					checked={enabled}
					onChange={(e) => setEnabled(e.target.checked)}
				/>
				有効にする
			</label>
			<Button type="submit" disabled={tooMany}>
				定期実行を保存
			</Button>
		</form>
	);
}

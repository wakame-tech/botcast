import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { $cms, type PodcastSchedule, recordToScript } from "@/lib/cms_client";
import { useState } from "react";

interface ScheduleFormProps {
	value?: PodcastSchedule;
	onSubmit: (schedule: PodcastSchedule) => void;
}

/** 番組の定期実行 (お便りエピソードの自動生成) の設定 */
export function ScheduleForm({ value, onSubmit }: ScheduleFormProps) {
	const [cron, setCron] = useState(value?.cron ?? "0 0 12 * * Fri *");
	const [scriptId, setScriptId] = useState(value?.script_id ?? "");
	const [enabled, setEnabled] = useState(value?.enabled ?? false);
	const [args, setArgs] = useState<Record<string, unknown>>(
		value?.arguments ?? {},
	);
	const { data: records } = $cms.useQuery("get", "/records/{collectionId}", {
		params: { path: { collectionId: "scripts" } },
	});
	const scripts = (records ?? []).map(recordToScript);
	const schema = scripts.find((s) => s.id === scriptId)?.arguments ?? {};

	return (
		<form
			className="flex flex-col gap-2 py-4"
			onSubmit={(e) => {
				e.preventDefault();
				onSubmit({ cron, script_id: scriptId, enabled, arguments: args });
			}}
		>
			<h2 className="text-lg font-bold">定期実行</h2>
			<label className="text-sm" htmlFor="schedule-cron">
				cron (秒 分 時 日 月 曜日 年、UTC)
			</label>
			<Input
				id="schedule-cron"
				value={cron}
				onChange={(e) => setCron(e.target.value)}
			/>
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
			<label className="text-sm flex items-center gap-2">
				<input
					type="checkbox"
					checked={enabled}
					onChange={(e) => setEnabled(e.target.checked)}
				/>
				有効にする
			</label>
			<Button type="submit">定期実行を保存</Button>
		</form>
	);
}

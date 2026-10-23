import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { Voice, VoiceParams } from "@/lib/cms_client";
import { useState } from "react";

/** フォームで編集する声のフィールド (ファイルのフィールドは別に扱う) */
export type VoiceInput = Pick<
	Voice,
	| "slug"
	| "name"
	| "engine"
	| "voice_id"
	| "styles"
	| "params"
	| "seed"
	| "caption"
>;

/** VoiceInput のキー (更新時に既存の data から消して入れ直す) */
export const VOICE_INPUT_KEYS = [
	"slug",
	"name",
	"engine",
	"voice_id",
	"styles",
	"params",
	"seed",
	"caption",
] as const;

/** CMS のスキーマの `slug` の pattern と同じ (数字だけの speaker は旧形式のスタイル ID) */
export const SLUG_PATTERN = "^[a-z][a-z0-9_\\-]*$";

const PARAM_KEYS = ["speed", "pitch", "intonation", "volume"] as const;

const PARAM_LABELS: Record<(typeof PARAM_KEYS)[number], string> = {
	speed: "話速",
	pitch: "音高",
	intonation: "抑揚",
	volume: "音量",
};

interface Row {
	key: number;
	name: string;
	value: string;
}

let nextRowKey = 0;
const toRows = (record: Record<string, string> | undefined): Row[] =>
	Object.entries(record ?? {}).map(([name, value]) => ({
		key: nextRowKey++,
		name,
		value,
	}));

const optionalNumber = (value: string): number | undefined =>
	value.trim() === "" ? undefined : Number(value);

const optionalString = (value: string): string | undefined =>
	value.trim() === "" ? undefined : value.trim();

/** undefined の値と空のオブジェクトを除いた data を作る */
export const voiceInputToData = (input: VoiceInput): Record<string, unknown> =>
	Object.fromEntries(
		Object.entries(input).filter(
			([, v]) =>
				v !== undefined &&
				!(typeof v === "object" && v !== null && Object.keys(v).length === 0),
		),
	);

interface VoiceFormProps {
	value?: VoiceInput;
	onSubmit: (input: VoiceInput) => void | Promise<void>;
}

/** 声の作成・編集のフォーム */
export function VoiceForm({ value, onSubmit }: VoiceFormProps) {
	const [slug, setSlug] = useState(value?.slug ?? "");
	const [name, setName] = useState(value?.name ?? "");
	const [engine, setEngine] = useState(value?.engine ?? "voicevox");
	const [voiceId, setVoiceId] = useState(value?.voice_id ?? "");
	const [caption, setCaption] = useState(value?.caption ?? "");
	const [seed, setSeed] = useState(value?.seed?.toString() ?? "");
	const [params, setParams] = useState<Record<string, string>>(
		Object.fromEntries(
			PARAM_KEYS.map((k) => [k, value?.params?.[k]?.toString() ?? ""]),
		),
	);
	const [extra, setExtra] = useState(
		value?.params?.extra ? JSON.stringify(value.params.extra, null, 2) : "",
	);
	const [styles, setStyles] = useState<Row[]>(toRows(value?.styles));
	const [error, setError] = useState<string>();

	const handleSubmit = async () => {
		let extraValue: Record<string, unknown> | undefined;
		if (extra.trim() !== "") {
			try {
				extraValue = JSON.parse(extra);
			} catch {
				setError("extra は JSON のオブジェクトで書いてください");
				return;
			}
		}
		const voiceParams: VoiceParams = {
			...Object.fromEntries(
				PARAM_KEYS.map((k) => [k, optionalNumber(params[k] ?? "")]).filter(
					([, v]) => v !== undefined,
				),
			),
			...(extraValue ? { extra: extraValue } : {}),
		};
		const stylesValue = Object.fromEntries(
			styles
				.filter((r) => r.name.trim() !== "" && r.value.trim() !== "")
				.map((r) => [r.name.trim(), r.value.trim()]),
		);
		setError(undefined);
		await onSubmit({
			slug: slug.trim(),
			name: name.trim(),
			engine: engine.trim(),
			voice_id: optionalString(voiceId),
			caption: optionalString(caption),
			seed: optionalNumber(seed),
			params: voiceParams,
			styles: stylesValue,
		});
	};

	return (
		<form
			className="flex flex-col gap-2 py-4"
			onSubmit={(e) => {
				e.preventDefault();
				handleSubmit().catch((err) => setError(String(err)));
			}}
		>
			<label className="text-sm" htmlFor="voice-slug">
				slug (台本の speaker・配役で使う
				ID。英小文字で始め、英小文字・数字・-・_)
			</label>
			<Input
				id="voice-slug"
				value={slug}
				onChange={(e) => setSlug(e.target.value)}
				pattern={SLUG_PATTERN}
				// 配役・台本は slug で声を参照するので、作成後は変えない
				readOnly={value !== undefined}
				required
			/>
			<label className="text-sm" htmlFor="voice-name">
				名前
			</label>
			<Input
				id="voice-name"
				value={name}
				onChange={(e) => setName(e.target.value)}
				required
			/>
			<label className="text-sm" htmlFor="voice-engine">
				エンジン (worker の TTS_ENGINES の名前。例: voicevox, irodori)
			</label>
			<Input
				id="voice-engine"
				value={engine}
				onChange={(e) => setEngine(e.target.value)}
				required
			/>
			<label className="text-sm" htmlFor="voice-voice-id">
				voice_id (VOICEVOX はスタイル ID、OpenAI 互換は voice
				名。参照音声をアップロードする場合だけ空でよい。参照音声が無いなら必須で、irodori
				で参照音声を使わないなら <code>none</code>)
			</label>
			<Input
				id="voice-voice-id"
				value={voiceId}
				onChange={(e) => setVoiceId(e.target.value)}
			/>
			<label className="text-sm" htmlFor="voice-caption">
				caption (声の説明。OpenAI 互換で style の無いセリフに使う)
			</label>
			<Input
				id="voice-caption"
				value={caption}
				onChange={(e) => setCaption(e.target.value)}
			/>
			<label className="text-sm" htmlFor="voice-seed">
				seed (同じ台本で同じ声にする。空ならエンジン任せ)
			</label>
			<Input
				id="voice-seed"
				type="number"
				step={1}
				value={seed}
				onChange={(e) => setSeed(e.target.value)}
			/>

			<fieldset className="text-sm flex flex-wrap gap-2">
				<legend>音質 (空ならエンジンの既定)</legend>
				{PARAM_KEYS.map((k) => (
					<div key={k}>
						<label htmlFor={`voice-param-${k}`}>
							{PARAM_LABELS[k]} ({k})
						</label>
						<Input
							id={`voice-param-${k}`}
							type="number"
							step="any"
							value={params[k]}
							onChange={(e) => setParams({ ...params, [k]: e.target.value })}
						/>
					</div>
				))}
			</fieldset>
			<label className="text-sm" htmlFor="voice-extra">
				extra (エンジンにそのまま渡す値。JSON のオブジェクト)
			</label>
			<Textarea
				id="voice-extra"
				value={extra}
				onChange={(e) => setExtra(e.target.value)}
			/>

			<fieldset className="text-sm flex flex-col gap-1">
				<legend>
					styles (VOICEVOX 互換: 台本の style の名前 → スタイル ID。無い style
					は既定のスタイル)
				</legend>
				{styles.map((row) => (
					<div key={row.key} className="flex gap-2 items-center">
						<Input
							placeholder="style の名前 (例: ささやき)"
							value={row.name}
							onChange={(e) =>
								setStyles(
									styles.map((r) =>
										r.key === row.key ? { ...r, name: e.target.value } : r,
									),
								)
							}
						/>
						<Input
							placeholder="スタイル ID"
							value={row.value}
							onChange={(e) =>
								setStyles(
									styles.map((r) =>
										r.key === row.key ? { ...r, value: e.target.value } : r,
									),
								)
							}
						/>
						<Button
							type="button"
							onClick={() => setStyles(styles.filter((r) => r.key !== row.key))}
						>
							削除
						</Button>
					</div>
				))}
				<div>
					<Button
						type="button"
						onClick={() =>
							setStyles([...styles, { key: nextRowKey++, name: "", value: "" }])
						}
					>
						style を追加
					</Button>
				</div>
			</fieldset>

			{error && <p className="text-sm text-red-500">{error}</p>}
			<Button type="submit">{value ? "更新" : "作成"}</Button>
		</form>
	);
}

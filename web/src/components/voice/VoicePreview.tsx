import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useVoiceFile } from "@/components/voice/VoiceAudio";
import {
	MAX_PREVIEW_TAKES,
	MAX_PREVIEW_TEXT_CHARS,
	createVoicePreviews,
} from "@/lib/worker_client";
import { useEffect, useRef, useState } from "react";

/**
 * 試聴の既定の本文 (worker の `DEFAULT_PREVIEW_TEXT` と同じ)。
 * 参照音声にするテイクは長いほうが声が安定する (Irodori-TTS は 30 秒ほど) ので数文にしている
 */
export const DEFAULT_PREVIEW_TEXT =
	"こんにちは。今日も番組を聞いてくださって、ありがとうございます。" +
	"最近は朝晩がすっかり涼しくなって、温かい飲み物がおいしい季節になりました。" +
	"この番組では、気になったニュースや身の回りの小さな発見を、のんびりとお話ししていきます。" +
	"どうぞ肩の力を抜いて、最後までゆっくりお付き合いください。";

interface Take {
	seed: number;
	blob: Blob;
	url: string;
	elapsedMs: number;
}

const base64ToBlob = (base64: string, contentType: string): Blob => {
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i);
	}
	return new Blob([bytes], { type: contentType });
};

interface VoicePreviewProps {
	voiceId: string;
	/** テイクを参照音声にするときに、声の seed をそのテイクの seed にする (ファイルのアップロードより先に呼ぶ) */
	onSetSeed: (seed: number) => Promise<void>;
}

/**
 * 声の試聴を作る (持ち主だけ)。同じ本文を seed を変えて何回か合成し、聞き比べて選んだテイクを
 * 参照音声 (ref_audio) か試聴用の音声 (sample) にする。テイクは保存されない (ページを離れると消える)
 */
export function VoicePreview({ voiceId, onSetSeed }: VoicePreviewProps) {
	const [text, setText] = useState(DEFAULT_PREVIEW_TEXT);
	const [count, setCount] = useState("3");
	const [takes, setTakes] = useState<Take[]>([]);
	const [progress, setProgress] = useState<{ done: number; total: number }>();
	const [busySeed, setBusySeed] = useState<number>();
	const [message, setMessage] = useState<string>();
	const [error, setError] = useState<string>();
	const refAudio = useVoiceFile(voiceId, "ref_audio");
	const sample = useVoiceFile(voiceId, "sample");
	const abort = useRef<AbortController>();
	const urls = useRef<string[]>([]);

	// ページを離れたら生成を止め、Blob URL を捨てる
	useEffect(
		() => () => {
			abort.current?.abort();
			for (const url of urls.current) URL.revokeObjectURL(url);
		},
		[],
	);

	const clearTakes = () => {
		for (const url of urls.current) URL.revokeObjectURL(url);
		urls.current = [];
		setTakes([]);
	};

	const generate = async () => {
		const total = Math.min(Math.max(Number(count) || 1, 1), MAX_PREVIEW_TAKES);
		setError(undefined);
		setMessage(undefined);
		clearTakes();
		const controller = new AbortController();
		abort.current = controller;
		// 1 テイクずつ頼む (1 回の HTTP を短くし、できたテイクから聞けるようにする)
		try {
			for (let i = 0; i < total; i++) {
				setProgress({ done: i, total });
				const res = await createVoicePreviews(
					voiceId,
					{ text: text.trim() || undefined, count: 1 },
					controller.signal,
				);
				const added = res.takes.map((t) => {
					const blob = base64ToBlob(t.audioBase64, t.contentType);
					const url = URL.createObjectURL(blob);
					urls.current.push(url);
					return { seed: t.seed, blob, url, elapsedMs: t.elapsedMs };
				});
				setTakes((prev) => [...prev, ...added]);
			}
		} catch (err) {
			if (!controller.signal.aborted) {
				setError(String(err));
			}
		} finally {
			setProgress(undefined);
		}
	};

	const applyTake = async (take: Take, field: "ref_audio" | "sample") => {
		setError(undefined);
		setMessage(undefined);
		setBusySeed(take.seed);
		try {
			const file = new File([take.blob], `preview-${take.seed}.wav`, {
				type: "audio/wav",
			});
			if (field === "ref_audio") {
				// seed の更新はレコードの data を置き換えるので、ファイルのアップロードより先に行う
				await onSetSeed(take.seed);
				const err = await refAudio.upload(file, "audio/wav", "");
				if (err) throw new Error(err);
				setMessage(
					`seed ${take.seed} のテイクを参照音声にし、声の seed を ${take.seed} にしました`,
				);
			} else {
				const err = await sample.upload(file, "audio/wav", "");
				if (err) throw new Error(err);
				setMessage(`seed ${take.seed} のテイクを試聴用の音声にしました`);
			}
		} catch (err) {
			setError(String(err));
		} finally {
			setBusySeed(undefined);
		}
	};

	const generating = progress !== undefined;
	const chars = [...text].length;

	return (
		<section className="flex flex-col gap-2 py-4">
			<h2 className="font-bold">試聴を作る</h2>
			<p className="text-sm text-gray-500">
				この声の設定 (エンジン・caption・params)
				で同じ文を何回か読ませ、テイクごとに seed
				を変えます。気に入ったテイクを参照音声にすると、以後はその声を複製して読むので声が安定します
				(参照音声があると caption
				は使いません)。参照音声にするなら長いほうがよく、20〜30
				秒ほど読める数文がおすすめです。
			</p>
			<label className="text-sm" htmlFor="voice-preview-text">
				本文 ({chars}/{MAX_PREVIEW_TEXT_CHARS}{" "}
				文字。絵文字は声が変わることがあるので避ける)
			</label>
			<Textarea
				id="voice-preview-text"
				rows={4}
				value={text}
				maxLength={MAX_PREVIEW_TEXT_CHARS}
				onChange={(e) => setText(e.target.value)}
				disabled={generating}
			/>
			<div className="flex gap-2 items-end">
				<div>
					<label className="text-sm" htmlFor="voice-preview-count">
						テイク数 (1〜{MAX_PREVIEW_TAKES})
					</label>
					<Input
						id="voice-preview-count"
						type="number"
						min={1}
						max={MAX_PREVIEW_TAKES}
						step={1}
						value={count}
						onChange={(e) => setCount(e.target.value)}
						disabled={generating}
					/>
				</div>
				<Button
					type="button"
					disabled={generating || chars > MAX_PREVIEW_TEXT_CHARS}
					onClick={() => {
						generate();
					}}
				>
					試聴を作る
				</Button>
				{generating && (
					<Button
						type="button"
						variant="outline"
						onClick={() => abort.current?.abort()}
					>
						中止
					</Button>
				)}
			</div>
			{progress && (
				<p className="text-sm text-gray-500" aria-live="polite">
					テイク {progress.done + 1}/{progress.total} を生成中... (1 テイクに
					10〜20 秒ほどかかります)
				</p>
			)}
			{takes.length > 0 && (
				<ol className="flex flex-col gap-2">
					{takes.map((take, i) => (
						<li
							key={take.seed}
							className="flex flex-wrap gap-2 items-center border rounded p-2"
						>
							<span className="text-sm">
								テイク {i + 1} (seed {take.seed}・
								{(take.elapsedMs / 1000).toFixed(1)} 秒で生成)
							</span>
							{/* biome-ignore lint/a11y/useMediaCaption: 試聴の音声に字幕は無い */}
							<audio controls src={take.url} />
							<Button
								type="button"
								size="sm"
								disabled={busySeed !== undefined}
								onClick={() => applyTake(take, "ref_audio")}
							>
								この声を参照音声にする
							</Button>
							<Button
								type="button"
								size="sm"
								variant="outline"
								disabled={busySeed !== undefined}
								onClick={() => applyTake(take, "sample")}
							>
								試聴用の音声にする
							</Button>
							{busySeed === take.seed && (
								<span className="text-sm text-gray-500">保存中...</span>
							)}
						</li>
					))}
				</ol>
			)}
			{message && <p className="text-sm text-green-700">{message}</p>}
			{error && <p className="text-sm text-red-500">{error}</p>}
		</section>
	);
}

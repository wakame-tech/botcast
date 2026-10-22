import type { Section } from "@/lib/api_client";
import type { Line } from "srt-parser-2";

/** worker が音声の区間 (Audio) の字幕に入れる文字 */
const AUDIO_SUBTITLE = "♪";

/** 0 秒も出す (formatMmss は 0 を未設定として扱う) */
const formatStart = (seconds: number) =>
	`${Math.floor(seconds / 60)}:${String(Math.floor(seconds % 60)).padStart(2, "0")}`;

const normalize = (text: string) => text.replace(/\s+/g, "");

const matches = (section: Section, text: string): boolean =>
	section.type === "Audio"
		? text === AUDIO_SUBTITLE
		: normalize(section.text).includes(text);

/**
 * 字幕の各行の話者を台本から引く。
 * worker はセリフを文に分けて字幕にする (1 セリフ → 複数行) ので、字幕の順に台本を先へ進めながら、行の文を含むセクションを探す
 */
export const speakersOf = (lines: Line[], sections: Section[]) => {
	let cursor = 0;
	return lines.map((line) => {
		const text = normalize(line.text);
		const found = sections.findIndex(
			(section, i) => i >= cursor && matches(section, text),
		);
		if (found === -1) {
			return undefined;
		}
		cursor = found;
		const section = sections[found];
		return section?.type === "Serif" ? section.speaker : undefined;
	});
};

interface TranscriptProps {
	lines: Line[];
	sections: Section[];
	/** プレイヤーがこのエピソードを再生 (一時停止) 中か */
	isCurrentEpisode: boolean;
	seconds: number;
	onClickLine: (seconds: number) => void;
}

/** 音声生成済みのエピソードの原稿。再生中の行を黒、それ以外を灰色にする */
export function Transcript(props: TranscriptProps) {
	const speakers = speakersOf(props.lines, props.sections);
	return (
		<ol className="m-0 flex list-none flex-col p-0">
			{props.lines.map((line, i) => {
				const active =
					props.isCurrentEpisode &&
					line.startSeconds <= props.seconds &&
					props.seconds < line.endSeconds;
				const color =
					active || !props.isCurrentEpisode ? "text-black" : "text-gray-400";
				return (
					<li key={line.id}>
						<button
							type="button"
							className={`flex w-full gap-3 rounded border-0 bg-transparent px-2 py-1 text-left hover:bg-gray-100 ${color} ${active ? "font-bold" : ""}`}
							onClick={() => props.onClickLine(line.startSeconds)}
						>
							<span className="w-12 shrink-0 font-mono text-sm leading-6 text-gray-400">
								{formatStart(line.startSeconds)}
							</span>
							{speakers[i] && (
								<span className="shrink-0 text-sm leading-6 text-teal-600">
									{speakers[i]}
								</span>
							)}
							<span className="leading-6">{line.text}</span>
						</button>
					</li>
				);
			})}
		</ol>
	);
}

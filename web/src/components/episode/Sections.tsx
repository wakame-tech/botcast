import type { Section } from "@/lib/api_client";
import type { Line } from "srt-parser-2";

interface ScriptLinesProps {
	lines: Line[];
	toBeHighlight: (line: Line) => boolean;
	onClick: (line: Line) => void;
}

export function SectionSentences(props: ScriptLinesProps) {
	return (
		<>
			{props.lines.map((line) => (
				<span
					key={`t${line.startSeconds}`}
					className={`${props.toBeHighlight(line) ? "bg-yellow-200" : "hover:bg-gray-200"} `}
					onClick={() => props.onClick(line)}
					onKeyDown={() => props.onClick(line)}
				>
					{line.text}
				</span>
			))}
		</>
	);
}

interface SectionsComponentProps {
	isPlayingEpisode: boolean;
	seconds: number;
	lines: Line[];
	sections: Section[];
	onClickLine: (seconds: number) => void;
}

export function SectionsComponent(props: SectionsComponentProps) {
	return (
		<div>
			<SectionSentences
				lines={props.lines}
				toBeHighlight={(line) =>
					props.isPlayingEpisode &&
					line.startSeconds <= props.seconds &&
					props.seconds < line.endSeconds
				}
				onClick={(line) => props.onClickLine(line.startSeconds)}
			/>
		</div>
	);
}

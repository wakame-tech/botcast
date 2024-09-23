import type { Line } from "srt-parser-2";

interface ScriptLinesProps {
	lines: Line[];
	toBeHighlight: (line: Line) => boolean;
	onClick: (line: Line) => void;
}

export function ScriptLines(props: ScriptLinesProps) {
	return (
		<>
			{props.lines.map((line) => (
				<div
					key={`t${line.startSeconds}`}
					className={`${props.toBeHighlight(line) ? "bg-yellow-100" : ""} hover:bg-gray-100`}
					onClick={() => props.onClick(line)}
					onKeyDown={() => props.onClick(line)}
				>
					<p className="p-0 m-0 text-lg py-2">{line.text}</p>
				</div>
			))}
		</>
	);
}

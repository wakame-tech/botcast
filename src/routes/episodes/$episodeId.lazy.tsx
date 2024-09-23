import { createLazyFileRoute } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";
import Parser from "srt-parser-2";
import type { Line } from "srt-parser-2";
import { useEffect, useState } from "react";
import { usePlayer } from "../../hooks/usePlayer";
import { UserIcon } from "../../components/user/UserIcon.tsx";

export const Route = createLazyFileRoute("/episodes/$episodeId")({
	component: Episode,
});

const fetchSrt = async (url: string): Promise<Line[]> => {
	const res = await fetch(url);
	if (!res.ok) {
		throw new Error(`Failed to fetch ${url}: ${res.status}`);
	}
	const srt = await res.text();
	const parser = new Parser();
	return parser.fromSrt(srt);
};

function Episode() {
	const { episodeId } = Route.useParams();
	const getEpisode = trpc.episode.useQuery({ id: episodeId });
	const deleteEpisode = trpc.deleteEpisode.useMutation();
	const episode = getEpisode.data?.episode;
	const [lines, setLines] = useState<Line[]>([]);
	const { isPlaying, play, seconds, seek, render } = usePlayer();

	useEffect(() => {
		(async () => {
			if (!episode || !episode?.script_url) {
				return;
			}
			const lines = await fetchSrt(episode.script_url);
			setLines(lines);
		})();
	}, [episode]);

	const handleDelete = async () => {
		await deleteEpisode.mutateAsync({ id: episodeId });
	};

	const handleSeek = (line: Line) => {
		seek(line.startSeconds);
	};

	if (!episode || !episode.user) {
		return <div>not found</div>;
	}

	return (
		<>
			<h1 className="text-2xl font-bold">{episode.title}</h1>

			<UserIcon user={episode.user} />

			<article className="pt-2">
				{lines.map((line) => (
					<div
						key={`t${line.startSeconds}`}
						className={`${line.startSeconds <= seconds && seconds < line.endSeconds ? "bg-yellow-100" : ""} hover:bg-gray-100`}
						onClick={() => handleSeek(line)}
						onKeyDown={() => handleSeek(line)}
					>
						<p className="p-0 m-0 text-lg py-2">{line.text}</p>
					</div>
				))}
			</article>

			<button type="button" onClick={handleDelete}>
				Delete
			</button>

			{episode.audio_url && (
				<div className="absolute sticky bottom-0 w-full h-15 bg-teal-100">
					<div className="p-2 flex justify-items-center">
						<button className="m-auto" type="button" onClick={() => play()}>
							{isPlaying ? "pause" : "play"}
						</button>
					</div>
					{render(episode.audio_url)}
				</div>
			)}
		</>
	);
}

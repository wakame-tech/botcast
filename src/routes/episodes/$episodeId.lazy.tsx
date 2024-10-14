import { ScriptLines } from "@/components/episode/ScriptLines.tsx";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { usePlayer } from "@/hooks/usePlayer";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";
import Parser from "srt-parser-2";
import type { Line } from "srt-parser-2";

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
	const navigate = useNavigate();
	const { episodeId } = Route.useParams();
	const getEpisode = trpc.episode.useQuery({ id: episodeId });
	const deleteEpisode = trpc.deleteEpisode.useMutation();
	const episode = getEpisode.data?.episode;
	const [lines, setLines] = useState<Line[]>([]);
	const { isPlaying, play, seconds, seek, render } = usePlayer();

	useEffect(() => {
		(async () => {
			if (!episode || !episode?.srt_url) {
				return;
			}
			const lines = await fetchSrt(episode.srt_url);
			setLines(lines);
		})();
	}, [episode]);

	if (!episode || !episode.user) {
		return <div>not found</div>;
	}

	const handleDelete = async () => {
		await deleteEpisode.mutateAsync({ id: episodeId });
		navigate({
			to: "/podcasts/$podcastId",
			params: { podcastId: episode.podcast_id },
		});
	};

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>{episode.title}</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="pb-2 flex items-center">
						<div className="flex-grow" />
						<Button className="bg-red-400" onClick={handleDelete}>
							削除
						</Button>
					</div>

					<ScriptLines
						lines={lines}
						toBeHighlight={(line) =>
							line.startSeconds <= seconds && seconds < line.endSeconds
						}
						onClick={(line) => seek(line.startSeconds)}
					/>
				</CardContent>
			</Card>

			{episode.audio_url && (
				<div className="sticky bottom-0 float-right p-2">
					<Button
						className="w-16 h-16 m-auto rounded-full"
						onClick={() => play()}
					>
						{isPlaying ? (
							<div className="i-solar:pause-outline w-2em h-2em" />
						) : (
							<div className="i-solar:play-outline w-2em h-2em" />
						)}
						<span>{render(episode.audio_url)}</span>
					</Button>
				</div>
			)}
		</>
	);
}

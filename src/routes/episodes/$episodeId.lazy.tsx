import { EpisodeComments } from "@/components/episode/EpisodeComments";
import { ScriptLines } from "@/components/episode/ScriptLines.tsx";
import { SectionsComponent } from "@/components/episode/Sections";
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
	const [lines, setLines] = useState<Line[]>([]);
	const { isPlaying, play, seconds, seek, render } = usePlayer();

	useEffect(() => {
		(async () => {
			if (!getEpisode.data?.episode.srt_url) {
				return;
			}
			const lines = await fetchSrt(getEpisode.data.episode.srt_url);
			setLines(lines);
		})();
	}, [getEpisode.data]);

	if (!getEpisode.data || getEpisode.error) {
		return <div>{getEpisode.error?.message}</div>;
	}

	const episode = getEpisode.data.episode;

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
					<Button onClick={handleDelete}>delete</Button>
					<SectionsComponent sections={episode.sections} />
					<ScriptLines
						lines={lines}
						toBeHighlight={(line) =>
							line.startSeconds <= seconds && seconds < line.endSeconds
						}
						onClick={(line) => seek(line.startSeconds)}
					/>

					{episode.audio_url ? (
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
					) : (
						<div className="flex items-center justify-center">
							<p className="text-gray-400 text-xl">音声ファイルがありません</p>
						</div>
					)}
				</CardContent>
			</Card>

			<EpisodeComments episodeId={episodeId} />
		</>
	);
}

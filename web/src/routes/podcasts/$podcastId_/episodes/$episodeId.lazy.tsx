import { SectionsComponent } from "@/components/episode/Sections";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { usePlayer } from "@/hooks/usePlayer";
import { $api } from "@/lib/api_client";
import { useQuery } from "@tanstack/react-query";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import Parser from "srt-parser-2";
import type { Line } from "srt-parser-2";

export const Route = createLazyFileRoute(
	"/podcasts/$podcastId/episodes/$episodeId",
)({
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
	const { podcastId, episodeId } = Route.useParams();
	const getEpisode = $api.useQuery("get", "/episodes/{episodeId}", {
		params: { path: { episodeId } },
	});
	const { data } = useQuery({
		queryKey: ["lines"],
		queryFn: () => {
			const url = getEpisode.data?.srt_url;
			if (url) {
				return fetchSrt(url);
			}
		},
		enabled: !!getEpisode.data?.srt_url,
	});
	const lines = data ?? [];

	const deleteEpisode = $api.useMutation("delete", "/episodes/{episodeId}");

	const { state, setEpisode, playOrPause, seekTo } = usePlayer();

	if (!getEpisode.data || getEpisode.error) {
		return <div>error</div>;
	}

	const episode = getEpisode.data;
	const isPlayingEpisode = state.isPlaying && episode.id === state.episode?.id;

	const handleDelete = async () => {
		await deleteEpisode.mutateAsync({ params: { path: { episodeId } } });
		navigate({
			to: "/podcasts/$podcastId",
			params: { podcastId },
		});
	};

	const icon = isPlayingEpisode ? "i-solar:pause-bold" : "i-solar:play-bold";
	const iconColor = episode ? "text-white" : "text-gray-300";

	const onClickPlayButton = () => {
		setEpisode(episode);
		playOrPause();
	};

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>
						<div className="flex content-center gap-2">
							<div
								className="bg-teal-400 w-16 h-16 rounded-xl flex items-center justify-center"
								onClick={() => onClickPlayButton()}
								onKeyDown={() => onClickPlayButton()}
							>
								<p className={`${icon} ${iconColor}`} />
							</div>
							<span className="grow pl-2 text-xl font-bold">
								{episode.title}
							</span>
							<div className="text-sm">
								<Button className="bg-red-500" onClick={handleDelete}>
									削除
								</Button>
							</div>
						</div>
					</CardTitle>
				</CardHeader>
				<CardContent>
					<div className="py-2">
						<SectionsComponent
							isPlayingEpisode={isPlayingEpisode}
							seconds={state.seconds}
							lines={lines}
							sections={episode.sections}
							onClickLine={seekTo}
						/>
					</div>
				</CardContent>
			</Card>
		</>
	);
}

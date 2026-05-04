import Episode from "@/components/episode/EpisodeList.tsx";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { $cms, recordToEpisode, recordToPodcast } from "@/lib/cms_client";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const { podcastId } = Route.useParams();
	const { data: podcastRecord } = $cms.useQuery(
		"get",
		"/records/{collectionId}/{recordId}",
		{ params: { path: { collectionId: "podcasts", recordId: podcastId } } },
	);
	const { data: episodeRecords } = $cms.useQuery(
		"get",
		"/records/{collectionId}",
		{ params: { path: { collectionId: "episodes" } } },
	);

	if (!podcastRecord) {
		return <div>not found</div>;
	}

	const podcast = recordToPodcast(podcastRecord);
	const episodes = (episodeRecords ?? [])
		.map(recordToEpisode)
		.filter((e) => e.podcast_id === podcastId);

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle className="flex-inline items-center gap-2">
						<span className="bg-teal-300 w-16 h-16 rounded-xl flex items-center justify-center">
							{podcast.icon}
						</span>
						<span className="grow pl-2 text-xl font-bold">{podcast.title}</span>
						<p className="text-sm">
							<Link
								to="/podcasts/$podcastId/edit"
								params={{ podcastId }}
								className="align-middle"
							>
								編集
							</Link>
						</p>
					</CardTitle>
				</CardHeader>
				<CardContent>
					<h2>概要</h2>
					{podcast.description}

					<h2>エピソード ({episodes.length})</h2>
					<Episode.List podcastId={podcast.id} episodes={episodes} />
				</CardContent>
			</Card>
		</>
	);
}

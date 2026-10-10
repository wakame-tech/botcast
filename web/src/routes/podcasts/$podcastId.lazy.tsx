import Episode from "@/components/episode/EpisodeList.tsx";
import { JobButton } from "@/components/job/JobButton";
import { JobList } from "@/components/job/JobList";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
	$cms,
	CMS_TENANT,
	recordToEpisode,
	recordToPodcast,
} from "@/lib/cms_client";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const { podcastId } = Route.useParams();
	const { data: podcastRecord } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
		{
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "podcasts",
					recordId: podcastId,
				},
			},
		},
	);
	const { data: episodeRecords } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: {
				path: { tenantId: CMS_TENANT, collectionId: "episodes" },
				query: {
					filter: [`podcast_id:eq:${podcastId}`],
					sort: "created_at",
					order: "desc",
				},
			},
		},
	);

	if (!podcastRecord) {
		return <div>not found</div>;
	}

	const podcast = recordToPodcast(podcastRecord);
	const episodes = (episodeRecords ?? []).map(recordToEpisode);

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

					<h2>ジョブ</h2>
					<JobButton
						label="お便りエピソードを生成"
						args={{ type: "generateEpisode", podcastId: podcast.id }}
						disabled={!podcast.schedule?.script_id}
					/>
					{!podcast.schedule?.script_id && (
						<p className="text-sm text-gray-500">
							番組のスケジュールにスクリプトを設定すると生成できます
						</p>
					)}
					<JobList />
				</CardContent>
			</Card>
		</>
	);
}

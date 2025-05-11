import { CornerList } from "@/components/corner/CornerList.tsx";
import Episode from "@/components/episode/EpisodeList.tsx";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserIcon } from "@/components/user/UserIcon";
import { $api } from "@/lib/api_client";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const { podcastId } = Route.useParams();
	const getPodcast = $api.useQuery('get', '/podcast/{podcastId}', {
		params: { path: { podcastId } }
	});

	if (!getPodcast.data) {
		return <div>not found</div>;
	}

	const { podcast, episodes, corners } = getPodcast.data;

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
					<UserIcon
						userId={podcast.user.auth_id}
						size="2rem"
						label={podcast.user.name ?? undefined}
					/>
					{podcast.description}

					<h2>コーナー ({corners.length})</h2>
					<CornerList corners={corners} />

					<h2>エピソード ({episodes.length})</h2>
					<Episode.List episodes={episodes} />
				</CardContent>
			</Card>
		</>
	);
}

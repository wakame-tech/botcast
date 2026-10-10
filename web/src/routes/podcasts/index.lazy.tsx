import Podcast from "@/components/podcast/PodcastList";
import { Button } from "@/components/ui/button";
import { $cms, CMS_TENANT, recordToPodcast } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/")({
	component: Podcasts,
});

export default function Podcasts() {
	const { data: records } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "podcasts" } },
		},
	);
	const podcasts = (records ?? []).map(recordToPodcast);

	return (
		<div>
			<div className="pb-2 flex items-center">
				<div className="flex-grow" />
				<Link to="/podcasts/new">
					<Button>新規作成</Button>
				</Link>
			</div>

			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

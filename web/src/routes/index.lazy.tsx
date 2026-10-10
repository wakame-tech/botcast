import Podcast from "@/components/podcast/PodcastList";
import { $cms, CMS_TENANT, recordToPodcast } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/")({
	component: Index,
});

function Index() {
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
			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

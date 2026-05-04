import Podcast from "@/components/podcast/PodcastList";
import { $cms, recordToPodcast } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/")({
	component: Index,
});

function Index() {
	const { data: records } = $cms.useQuery("get", "/records/{collectionId}", {
		params: { path: { collectionId: "podcasts" } },
	});
	const podcasts = (records ?? []).map(recordToPodcast);

	return (
		<div>
			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

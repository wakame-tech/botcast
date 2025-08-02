import Podcast from "@/components/podcast/PodcastList";
import { $api } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/")({
	component: Index,
});

function Index() {
	const getTopPodcasts = $api.useQuery("get", "/topPodcasts");
	const podcasts = getTopPodcasts.data ?? [];

	return (
		<div>
			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

import Podcast from "@/components/podcast/PodcastList";
import { trpc } from "@/trpc";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/")({
	component: Index,
});

function Index() {
	const getTopPodcasts = trpc.topPodcasts.useQuery();
	const podcasts = getTopPodcasts.data?.podcasts ?? [];

	return (
		<div>
			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

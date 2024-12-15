import Podcast from "@/components/podcast/PodcastList";
import { trpc } from "@/trpc";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/")({
	component: Podcasts,
});

export default function Podcasts() {
	const getPodcasts = trpc.podcasts.useQuery();
	const podcasts = getPodcasts.data?.podcasts ?? [];

	return (
		<div>
			<Podcast.List podcasts={podcasts} />
		</div>
	);
}

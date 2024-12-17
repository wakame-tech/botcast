import Podcast from "@/components/podcast/PodcastList";
import { Button } from "@/components/ui/button";
import { trpc } from "@/trpc";
import { createLazyFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/")({
	component: Podcasts,
});

export default function Podcasts() {
	const getPodcasts = trpc.podcasts.useQuery();
	const podcasts = getPodcasts.data?.podcasts ?? [];

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

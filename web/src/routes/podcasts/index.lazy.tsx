import Podcast from "@/components/podcast/PodcastList";
import { Button } from "@/components/ui/button";
import { $cms, recordToPodcast } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/")({
	component: Podcasts,
});

export default function Podcasts() {
	const { data: records } = $cms.useQuery("get", "/records/{collectionId}", {
		params: { path: { collectionId: "podcasts" } },
	});
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

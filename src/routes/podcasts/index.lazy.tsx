import Podcast from "@/components/podcast/PodcastList";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
			<Card>
				<CardHeader>
					<CardTitle>ポッドキャスト</CardTitle>
				</CardHeader>
				<CardContent>
					<Podcast.List podcasts={podcasts} />
				</CardContent>
			</Card>
		</div>
	);
}

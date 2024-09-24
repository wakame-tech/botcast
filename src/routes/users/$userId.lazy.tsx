import Podcast from "@/components/podcast/PodcastList";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { trpc } from "@/trpc";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/users/$userId")({
	component: User,
});

function User() {
	const getPodcasts = trpc.podcasts.useQuery();
	const podcasts = getPodcasts.data?.podcasts ?? [];

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>ポッドキャスト</CardTitle>
				</CardHeader>
				<CardContent>
					<Podcast.List podcasts={podcasts} />
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>タスク</CardTitle>
				</CardHeader>
				<CardContent>
					<Link to="/tasks">Tasks</Link>
				</CardContent>
			</Card>
		</>
	);
}

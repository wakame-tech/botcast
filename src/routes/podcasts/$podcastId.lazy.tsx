import Episode from "@/components/episode/EpisodeList.tsx";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserIcon } from "@/components/user/UserIcon";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const navigate = useNavigate();
	const { podcastId } = Route.useParams();
	const getPodcast = trpc.podcast.useQuery({ id: podcastId });
	const deletePodcast = trpc.deletePodcast.useMutation();
	const podcast = getPodcast.data?.podcast;

	if (!podcast || !podcast.user) {
		return <div>not found</div>;
	}

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({ id: podcastId });
		navigate({ to: "/podcasts" });
	};

	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>
						{podcast.icon} {podcast.title}
					</CardTitle>
					<UserIcon
						userId={podcast.user.auth_id}
						size="2rem"
						label={podcast.user.name ?? undefined}
					/>
				</CardHeader>
				<CardContent>
					<Episode.List
						podcastId={podcastId}
						episodes={podcast.episodes}
						onClickDelete={handleDelete}
					/>
				</CardContent>
			</Card>
		</>
	);
}

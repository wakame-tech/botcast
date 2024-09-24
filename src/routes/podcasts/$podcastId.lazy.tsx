import Episode from "@/components/episode/EpisodeList.tsx";
import { UserIcon } from "@/components/user/UserIcon";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const { podcastId } = Route.useParams();
	const getPodcast = trpc.podcast.useQuery({ id: podcastId });
	const deletePodcast = trpc.deletePodcast.useMutation();
	const podcast = getPodcast.data?.podcast;

	if (!podcast || !podcast.user) {
		return <div>not found</div>;
	}

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({ id: podcastId });
	};

	return (
		<>
			<span className="text-2xl font-bold pr-2">
				{podcast.icon} {podcast.title}
			</span>
			<UserIcon userId={podcast.user.auth_id} label={podcast.user.name ?? undefined} />

			<Episode.List
				podcastId={podcastId}
				episodes={podcast.episodes}
				onClickDelete={handleDelete}
			/>
		</>
	);
}

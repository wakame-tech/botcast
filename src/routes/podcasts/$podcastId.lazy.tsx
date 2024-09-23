import Episode from "@/components/episode/EpisodeList.tsx";
import { UserIcon } from "@/components/user/UserIcon.tsx";
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
			<div>
				<span className="text-2xl font-bold pr-2">
					{podcast.icon} {podcast.title}
				</span>
				<UserIcon user={podcast.user} />
			</div>

			<Episode.List
				podcastId={podcastId}
				episodes={podcast.episodes}
				onClickDelete={handleDelete}
			/>
		</>
	);
}

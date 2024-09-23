import { createLazyFileRoute, Link } from "@tanstack/react-router";
import { trpc } from "../../trpc.ts";
import dayjs from "dayjs";
import { UserIcon } from "../../components/user/UserIcon.tsx";

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

			<div className="pb-2 flex justify-end">
				<Link
					to="/podcasts/$podcastId/new"
					params={{ podcastId }}
					className="p-2 no-underline rounded-md bg-teal-500 text-white"
				>
					新規作成
				</Link>
			</div>

			{podcast.episodes.map((episode, i) => (
				<div key={episode.id} className="p-2 bg-teal-100">
					<Link
						to="/episodes/$episodeId"
						params={{ episodeId: episode.id }}
						className="no-underline"
					>
						<span className="text-xl text-gray pr-2">#{i + 1}</span>
						<span className="text-xl font-bold">{episode.title}</span>
					</Link>
					<p className="p-0 m-0">
						<span className="text-xs text-gray">
							{dayjs(episode.created_at).format("YYYY-MM-DD HH:mm")}
						</span>
					</p>
				</div>
			))}
			<button type="button" onClick={handleDelete}>
				Delete
			</button>
		</>
	);
}

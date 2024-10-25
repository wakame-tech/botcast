import Episode from "@/components/episode/EpisodeList.tsx";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { UserIcon } from "@/components/user/UserIcon";
import { trpc } from "@/trpc.ts";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId")({
	component: Podcast,
});

export function Podcast() {
	const navigate = useNavigate();
	const { podcastId } = Route.useParams();
	const getPodcast = trpc.podcast.useQuery({ id: podcastId });
	const deletePodcast = trpc.deletePodcast.useMutation();
	const addTask = trpc.addTask.useMutation();
	const podcast = getPodcast.data?.podcast;

	if (!podcast || !podcast.user) {
		return <div>not found</div>;
	}

	const handleDispatchNewEpisodeTask = async () => {
		await addTask.mutateAsync({
			type: "newEpisode",
			podcastId,
		});
	};

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
					<div>
						<Link
							to="/scripts/$scriptId/edit"
							params={{ scriptId: podcast.script_id }}
						>
							スクリプトを編集
						</Link>
					</div>
					<Button onClick={handleDispatchNewEpisodeTask}>
						エピソード作成を実行
					</Button>
					{/* @ts-ignore */}
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

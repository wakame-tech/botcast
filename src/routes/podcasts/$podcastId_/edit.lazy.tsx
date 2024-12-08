import { PodcastForm } from "@/components/podcast/PodcastForm";
import { Button } from "@/components/ui/button";
import type { PodcastInput } from "@/trpc.ts";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId/edit")({
	component: EditPodcast,
});

export default function EditPodcast() {
	const { podcastId } = Route.useParams();
	const navigate = Route.useNavigate();
	const getPodcast = trpc.podcast.useQuery({ id: podcastId });
	const updatePodcast = trpc.updatePodcast.useMutation();
	const deletePodcast = trpc.deletePodcast.useMutation();

	const handleSubmit = async (values: PodcastInput) => {
		await updatePodcast.mutateAsync({
			id: podcastId,
			title: values.title,
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({ id: podcastId });
		navigate({ to: "/podcasts" });
	};

	if (!getPodcast.data) {
		return null;
	}

	const podcast = getPodcast.data.podcast;

	return (
		<>
			<PodcastForm
				onSubmit={handleSubmit}
				values={{
					icon: podcast.icon,
					title: podcast.title,
					description: podcast.description,
				}}
			/>

			<div className="flex items-center">
				<div className="flex-grow" />
				<Button className="bg-red-400" onClick={handleDelete}>
					削除
				</Button>
			</div>
		</>
	);
}

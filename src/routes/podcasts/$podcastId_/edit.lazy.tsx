import type { PodcastEditFormValues } from "@/components/podcast/PodcastForm";
import { PodcastForm } from "@/components/podcast/PodcastForm";
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

	const handleSubmit = async (values: PodcastEditFormValues) => {
		await updatePodcast.mutateAsync({
			id: podcastId,
			title: values.title,
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
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
					title: podcast.title,
				}}
			/>
		</>
	);
}

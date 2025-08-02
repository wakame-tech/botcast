import { PodcastForm } from "@/components/podcast/PodcastForm";
import { Button } from "@/components/ui/button";
import { $api, type PodcastInput } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId/edit")({
	component: EditPodcast,
});

export default function EditPodcast() {
	const { podcastId } = Route.useParams();
	const navigate = Route.useNavigate();
	const getPodcast = $api.useQuery("get", "/podcast/{podcastId}", {
		params: { path: { podcastId } },
	});
	const updatePodcast = $api.useMutation("put", "/podcast/{podcastId}");
	const deletePodcast = $api.useMutation("delete", "/podcast/{podcastId}");

	const handleSubmit = async (values: PodcastInput) => {
		await updatePodcast.mutateAsync({
			params: {
				path: { podcastId },
			},
			body: {
				icon: values.icon,
				title: values.title,
				description: values.description,
			},
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({ params: { path: { podcastId } } });
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

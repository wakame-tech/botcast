import { PodcastForm } from "@/components/podcast/PodcastForm";
import { Button } from "@/components/ui/button";
import type { PodcastInput } from "@/lib/api_client";
import { $cms, recordToPodcast, toRecordData } from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId/edit")({
	component: EditPodcast,
});

export default function EditPodcast() {
	const { podcastId } = Route.useParams();
	const navigate = Route.useNavigate();
	const { data: podcastRecord } = $cms.useQuery(
		"get",
		"/records/{collectionId}/{recordId}",
		{ params: { path: { collectionId: "podcasts", recordId: podcastId } } },
	);
	const updatePodcast = $cms.useMutation(
		"put",
		"/records/{collectionId}/{recordId}",
	);
	const deletePodcast = $cms.useMutation(
		"delete",
		"/records/{collectionId}/{recordId}",
	);

	const handleSubmit = async (values: PodcastInput) => {
		await updatePodcast.mutateAsync({
			params: { path: { collectionId: "podcasts", recordId: podcastId } },
			body: {
				data: toRecordData({
					icon: values.icon,
					title: values.title,
					description: values.description,
				}),
			},
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({
			params: { path: { collectionId: "podcasts", recordId: podcastId } },
		});
		navigate({ to: "/podcasts" });
	};

	if (!podcastRecord) {
		return null;
	}

	const podcast = recordToPodcast(podcastRecord);

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

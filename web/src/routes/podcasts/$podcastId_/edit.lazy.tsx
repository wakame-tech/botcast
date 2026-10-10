import { PodcastForm } from "@/components/podcast/PodcastForm";
import { ScheduleForm } from "@/components/podcast/ScheduleForm";
import { Button } from "@/components/ui/button";
import type { PodcastInput } from "@/lib/api_client";
import {
	$cms,
	CMS_TENANT,
	type PodcastSchedule,
	recordToPodcast,
	toRecordData,
} from "@/lib/cms_client";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/podcasts/$podcastId/edit")({
	component: EditPodcast,
});

export default function EditPodcast() {
	const { podcastId } = Route.useParams();
	const navigate = Route.useNavigate();
	const { data: podcastRecord } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
		{
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "podcasts",
					recordId: podcastId,
				},
			},
		},
	);
	const updatePodcast = $cms.useMutation(
		"put",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);
	const deletePodcast = $cms.useMutation(
		"delete",
		"/t/{tenantId}/records/{collectionId}/{recordId}",
	);

	// data は丸ごと置き換わるため、既存の値 (user_id / schedule など) を保って更新する
	const saveData = async (patch: Record<string, unknown>) => {
		const current = (podcastRecord?.data ?? {}) as Record<string, unknown>;
		await updatePodcast.mutateAsync({
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "podcasts",
					recordId: podcastId,
				},
			},
			body: { data: toRecordData({ ...current, ...patch }) },
		});
	};

	const handleSubmit = async (values: PodcastInput) => {
		await saveData({
			icon: values.icon,
			title: values.title,
			description: values.description,
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleScheduleSubmit = async (schedule: PodcastSchedule) => {
		await saveData({ schedule });
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleDelete = async () => {
		await deletePodcast.mutateAsync({
			params: {
				path: {
					tenantId: CMS_TENANT,
					collectionId: "podcasts",
					recordId: podcastId,
				},
			},
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

			<ScheduleForm value={podcast.schedule} onSubmit={handleScheduleSubmit} />

			<div className="flex items-center">
				<div className="flex-grow" />
				<Button className="bg-red-400" onClick={handleDelete}>
					削除
				</Button>
			</div>
		</>
	);
}

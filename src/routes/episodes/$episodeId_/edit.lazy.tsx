import { EpisodeForm } from "@/components/episode/EpisodeForm";
import type { EpisodeEditFormValues } from "@/components/episode/EpisodeForm";
import { Button } from "@/components/ui/button";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useEffect, useState } from "react";

export const Route = createLazyFileRoute("/episodes/$episodeId/edit")({
	component: EditEpisode,
});

const useTask = () => {
	const [taskId, setTaskId] = useState<string | null>(null);
	const getTask = trpc.task.useQuery(
		// biome-ignore lint/style/noNonNullAssertion: conditional fetch
		{ id: taskId! },
		{ enabled: taskId !== null },
	);

	useEffect(() => {
		const id = setInterval(async () => {
			if (!taskId) {
				return;
			}
			const res = await getTask.refetch();
			if (res.error || !res.data?.task) {
				console.error(res.error);
				return;
			}
			const task = res.data?.task;
			if (task.status === "COMPLETED" || task.status === "FAILED") {
				setTaskId(null);
				clearInterval(id);
			}
		}, 3000);
		return () => clearInterval(id);
	}, [taskId, getTask]);

	return {
		taskId,
		setTaskId,
	};
};

type Section = {
	type: "Serif";
	speaker: string;
	text: string;
};

interface Manuscript {
	title: string;
	sections: Section[];
}

function ManuscriptComponent({ manuscript }: { manuscript: Manuscript }) {
	return (
		<div>
			<p className="font-bold text-lg">title: {manuscript.title}</p>
			{manuscript.sections.map((section) => (
				<p key={section.text}>
					<span className="pr-2 text-small text-gray">{section.speaker}</span>
					<span>{section.text}</span>
				</p>
			))}
		</div>
	);
}

export function EditEpisode() {
	const { episodeId } = Route.useParams();
	const navigate = useNavigate();
	const getEpisode = trpc.episode.useQuery({ id: episodeId });
	const deleteEpisode = trpc.deleteEpisode.useMutation();
	const updateScript = trpc.updateScript.useMutation();
	const addTask = trpc.addTask.useMutation();
	const { taskId, setTaskId } = useTask();

	useEffect(() => {
		if (!taskId) {
			getEpisode.refetch();
		}
	}, [getEpisode, taskId]);

	const handleDelete = async () => {
		await deleteEpisode.mutateAsync({ id: episodeId });
		navigate({
			to: "/podcasts/$podcastId",
			params: { podcastId: episode.podcast_id },
		});
	};

	const handleSubmit = async (values: EpisodeEditFormValues) => {
		if (!getEpisode.data) {
			return;
		}
		const { episode } = getEpisode.data;
		await updateScript.mutateAsync({
			id: episode.script.id,
			template: values.template,
		});
		const { task } = await addTask.mutateAsync({
			type: "evaluateScript",
			episodeId: episode.id,
		});
		console.log(task);
		setTaskId(task.id);
	};

	const handleGenerateAudio = async () => {
		const { task } = await addTask.mutateAsync({
			type: "generateAudio",
			episodeId: episode.id,
		});
		console.log(task);
		setTaskId(task.id);
	};

	if (!getEpisode.data) {
		return null;
	}

	const { episode } = getEpisode.data;
	// @ts-ignore
	const template = JSON.stringify(episode.script.template);

	return (
		<>
			<h1>Edit Episode</h1>
			<div className="pb-2 flex items-center">
				<div className="flex-grow" />
				<Button className="bg-red-400" onClick={handleDelete}>
					削除
				</Button>
			</div>

			<EpisodeForm
				disabled={taskId !== null}
				episodeId={episodeId}
				template={template}
				onSubmit={handleSubmit}
			/>
			<h2>Manuscript Preview</h2>
			<ManuscriptComponent
				manuscript={episode.manuscript as object as Manuscript}
			/>
			<Button disabled={taskId !== null} onClick={handleGenerateAudio}>
				音声生成
			</Button>
			<p>audio_url: {episode.audio_url ?? "null"}</p>
		</>
	);
}

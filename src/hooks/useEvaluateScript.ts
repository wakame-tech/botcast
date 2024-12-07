import { trpc } from "@/trpc.ts";
import { useEffect, useState } from "react";

export const useEvaluateScript = () => {
	const [taskId, setTaskId] = useState<string | null>(null);
	const [taskResult, setTaskResult] = useState<Record<string, unknown> | null>(
		{},
	);
	const addTask = trpc.addTask.useMutation();
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
			console.log(task);
			if (task.status === "COMPLETED" || task.status === "FAILED") {
				setTaskId(null);
				setTaskResult(task.result);
				clearInterval(id);
			}
		}, 3000);
		return () => clearInterval(id);
	}, [taskId, getTask]);

	const evaluate = async (template: Record<string, unknown>) => {
		const { task } = await addTask.mutateAsync({
			cron: null,
			args: {
				type: "evaluateTemplate",
				template: JSON.parse(JSON.stringify(template)),
				context: {},
			},
		});
		setTaskId(task.id);
	};

	return {
		taskId,
		taskResult,
		evaluate,
		running: taskId !== null,
	};
};

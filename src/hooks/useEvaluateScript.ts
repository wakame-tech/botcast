import { trpc } from "@/trpc.ts";
import { useEffect, useState } from "react";

export const useEvaluateScript = () => {
	const [taskId, setTaskId] = useState<string | null>(null);
	const [taskResult, setTaskResult] = useState<object>({});
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
				// @ts-ignore
				setTaskResult(JSON.stringify(task.result));
				clearInterval(id);
			}
		}, 3000);
		return () => clearInterval(id);
	}, [taskId, getTask]);

	const evaluate = async (template: string) => {
		const { task } = await addTask.mutateAsync({
			type: "evaluateTemplate",
			template: JSON.parse(template),
			context: {},
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

import { EvaluateScriptForm } from "@/components/task/EvaluateScriptForm";
import type { EvaluateScriptInput } from "@/components/task/EvaluateScriptForm";
import { TaskList } from "@/components/task/TaskList.tsx";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/tasks")({
	component: Tasks,
});

function Tasks() {
	const [includesCompleted, setIncludesCompleted] = useState(true);
	const [includesResult, setIncludesResult] = useState(false);
	const getTasks = trpc.tasks.useQuery({ includesCompleted });
	const deleteTask = trpc.deleteTask.useMutation();
	const addEvaluateTemplateTaskById =
		trpc.addEvaluateTemplateTaskById.useMutation();

	const onSubmit = async (values: EvaluateScriptInput) => {
		await addEvaluateTemplateTaskById.mutateAsync({
			id: values.scriptId,
			cron: values.cron,
		});
	};

	const onClickDelete = async (taskId: string) => {
		await deleteTask.mutateAsync({ id: taskId });
		getTasks.refetch();
	};

	return (
		<div>
			<EvaluateScriptForm onSubmit={onSubmit} />

			<div className="grid grid-cols-1 gap-2">
				<label>
					<input
						type="checkbox"
						checked={includesCompleted}
						onChange={(e) => setIncludesCompleted(e.target.checked)}
					/>
					<span>完了したタスクも表示</span>
				</label>
				<label>
					<input
						type="checkbox"
						checked={includesResult}
						onChange={(e) => setIncludesResult(e.target.checked)}
					/>
					<span>結果も表示</span>
				</label>
			</div>

			<TaskList
				onClickDelete={onClickDelete}
				includesResult={includesResult}
				tasks={getTasks.data?.tasks ?? []}
			/>
		</div>
	);
}

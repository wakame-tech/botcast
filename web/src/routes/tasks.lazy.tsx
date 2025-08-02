import { TaskList } from "@/components/task/TaskList.tsx";
import { $api } from "@/lib/api_client";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/tasks")({
	component: Tasks,
});

function Tasks() {
	const [includesCompleted, setIncludesCompleted] = useState(true);
	const [includesResult, setIncludesResult] = useState(false);
	const getTasks = $api.useQuery("get", "/tasks");
	const deleteTask = $api.useMutation("delete", "/tasks/{taskId}");

	const onClickDelete = async (taskId: string) => {
		await deleteTask.mutateAsync({ params: { path: { taskId } } });
		getTasks.refetch();
	};

	return (
		<div>
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
				tasks={getTasks.data ?? []}
			/>
		</div>
	);
}

import { TaskList } from "@/components/task/TaskList.tsx";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";
import { useEffect } from "react";

export const Route = createLazyFileRoute("/tasks")({
	component: Tasks,
});

function Tasks() {
	const getTasks = trpc.tasks.useQuery();

	useEffect(() => {
		const id = setInterval(() => {
			getTasks.refetch();
		}, 3000);
		return () => clearInterval(id);
	}, [getTasks]);

	return (
		<div>
			<TaskList tasks={getTasks.data?.tasks ?? []} />
		</div>
	);
}

import { TaskList } from "@/components/task/TaskList.tsx";
import { trpc } from "@/trpc.ts";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/tasks")({
	component: Tasks,
});

function Tasks() {
	const getTasks = trpc.tasks.useQuery();

	return (
		<div>
			<TaskList tasks={getTasks.data?.tasks ?? []} />
		</div>
	);
}

import type { Task } from "@prisma/client";

interface TaskListProps {
	tasks: Task[];
}

export function TaskList(props: TaskListProps) {
	return (
		<>
			{props.tasks.map((task) => (
				<div key={task.id} className="p-2 grid grid-cols-2">
					<span className="pr-2">{task.id}</span>
					<span className="font-bold">{task.status}</span>
				</div>
			))}
		</>
	);
}

const components = {
	List: TaskList,
};

export default components;

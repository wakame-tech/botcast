import type { Task, WithSerializedDates } from "@/trpc";
import dayjs from "dayjs";

interface TaskItemProps {
	task: WithSerializedDates<Task>;
}

export function TaskItem(props: TaskItemProps) {
	return (
		<>
			<div className="flex gap-2">
				<span className="pr-2 font-bold">{props.task.args.type}</span>
				<span className="pr-2 text-sm text-gray">
					{props.task.id.substring(0, 7)}
				</span>
				<span className="font-bold">{props.task.status}</span>
				<span className="text-sm text-gray">
					{props.task.execute_after &&
						dayjs(props.task.execute_after).format("YYYY-MM-DD HH:mm")}
				</span>
				{props.task.executed_at && props.task.executed_finished_at && (
					<span>
						{dayjs(props.task.executed_finished_at).diff(
							dayjs(props.task.executed_at).format("YYYY-MM-DD HH:mm"),
							"seconds",
						)}
						s
					</span>
				)}
				<span className="ml-2 text-sm text-gray">
					{JSON.stringify(props.task.args)}
				</span>
			</div>
			<span className="ml-2 text-sm text-gray">
				{JSON.stringify(props.task.result)}
			</span>
		</>
	);
}

interface TaskListProps {
	tasks: WithSerializedDates<Task>[];
}

export function TaskList(props: TaskListProps) {
	return (
		<>
			{props.tasks.map((task) => (
				<div key={task.id} className="p-2">
					<TaskItem task={task} />
				</div>
			))}
		</>
	);
}

const components = {
	List: TaskList,
	Item: TaskItem,
};

export default components;

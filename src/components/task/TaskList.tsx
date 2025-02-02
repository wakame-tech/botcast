import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import type { Task, WithSerializedDates } from "@/trpc";
import dayjs from "dayjs";
import type { TaskStatus } from "prisma_client";

function TaskStatusIcon({ status }: { status: TaskStatus }) {
	const icons: Record<TaskStatus, string> = {
		PENDING: "text-gray i-solar:clock-circle-bold",
		RUNNING: "text-gray i-solar:menu-dots-bold",
		COMPLETED: "text-green i-solar:check-circle-bold",
		FAILED: "text-red i-solar:close-circle-bold",
	};
	const icon = icons[status];
	return <div className={`${icon} h-4 w-4`} />;
}

interface TaskItemProps {
	includesResult?: boolean;
	task: WithSerializedDates<Task>;
	onClickDelete: (taskId: string) => void;
}

export function TaskItem(props: TaskItemProps) {
	return (
		<>
			<div className="flex gap-2">
				<span className="">
					<TaskStatusIcon status={props.task.status} />
				</span>
				<span className="pr-2 font-bold">{props.task.args.type}</span>
				<span className="pr-2 text-sm text-gray">
					{props.task.id.substring(0, 7)}
				</span>
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
				{props.task.cron && (
					<span className="text-sm text-gray">{props.task.cron}</span>
				)}
				<DropdownMenu>
					<DropdownMenuTrigger className="p-2 border-0">
						<div className="i-solar:menu-dots-bold h-4 w-4" />
					</DropdownMenuTrigger>
					<DropdownMenuContent>
						<DropdownMenuItem
							onClick={(_) => props.onClickDelete(props.task.id)}
							className="text-red"
						>
							削除
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			</div>
			{props.includesResult && (
				<div>
					<span className="ml-2 text-sm text-gray">
						{JSON.stringify(props.task.args)}
					</span>
					<span className="ml-2 text-sm text-gray">
						{JSON.stringify(props.task.result)}
					</span>
				</div>
			)}
		</>
	);
}

interface TaskListProps {
	includesResult?: boolean;
	tasks: WithSerializedDates<Task>[];
	onClickDelete: (taskId: string) => void;
}

export function TaskList(props: TaskListProps) {
	return (
		<>
			{props.tasks.map((task) => (
				<div key={task.id} className="p-2">
					<TaskItem
						onClickDelete={props.onClickDelete}
						includesResult={props.includesResult}
						task={task}
					/>
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

import { Button } from "@/components/ui/button";
import { type JobArgs, useEnqueueJob } from "@/lib/worker_client";

interface JobButtonProps {
	args: JobArgs;
	label: string;
	disabled?: boolean;
}

/** worker にジョブを投入するボタンと結果表示 */
export function JobButton({ args, label, disabled }: JobButtonProps) {
	const enqueue = useEnqueueJob();
	return (
		<div className="flex items-center gap-2">
			<Button
				type="button"
				disabled={disabled || enqueue.isPending}
				onClick={() => enqueue.mutate(args)}
			>
				{label}
			</Button>
			{enqueue.isSuccess && (
				<span className="text-sm text-teal-600">ジョブを投入しました</span>
			)}
			{enqueue.isError && (
				<span className="text-sm text-red-500">{enqueue.error.message}</span>
			)}
		</div>
	);
}

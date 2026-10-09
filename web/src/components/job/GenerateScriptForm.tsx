import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useEnqueueJob } from "@/lib/worker_client";
import { useState } from "react";

interface GenerateScriptFormProps {
	episodeId: string;
}

/** プロンプトを入力して台本生成ジョブを投入するフォーム */
export function GenerateScriptForm({ episodeId }: GenerateScriptFormProps) {
	const [prompt, setPrompt] = useState("");
	const enqueue = useEnqueueJob();

	const handleSubmit = (e: React.FormEvent) => {
		e.preventDefault();
		enqueue.mutate({ type: "generateScript", episodeId, prompt });
	};

	return (
		<form onSubmit={handleSubmit} className="flex flex-col gap-2">
			<Textarea
				placeholder="台本生成のプロンプト"
				value={prompt}
				onChange={(e) => setPrompt(e.target.value)}
				required
			/>
			<div className="flex items-center gap-2">
				<Button type="submit" disabled={enqueue.isPending}>
					台本を生成
				</Button>
				{enqueue.isSuccess && (
					<span className="text-sm text-teal-600">ジョブを投入しました</span>
				)}
				{enqueue.isError && (
					<span className="text-sm text-red-500">{enqueue.error.message}</span>
				)}
			</div>
		</form>
	);
}

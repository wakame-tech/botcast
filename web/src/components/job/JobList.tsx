import { useJobs } from "@/lib/worker_client";

/** worker のジョブ一覧 (`GET /jobs`) */
export function JobList() {
	const { data: jobs, error } = useJobs();

	if (error) {
		return <p className="text-sm text-red-500">{error.message}</p>;
	}

	return (
		<table className="w-full text-sm">
			<thead>
				<tr className="text-left">
					<th>ID</th>
					<th>名前</th>
					<th>状態</th>
				</tr>
			</thead>
			<tbody>
				{(jobs ?? []).map((job) => (
					<tr key={job.id}>
						<td className="font-mono">{job.id}</td>
						<td>{job.name}</td>
						<td>{job.status}</td>
					</tr>
				))}
			</tbody>
		</table>
	);
}

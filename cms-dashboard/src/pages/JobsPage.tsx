// @specre 01KNM2BBT5WCZP0PCD2F06CAES
import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { jobsApi } from "../api/client";
import type { CreateOrUpdateJob, Job } from "../types";
import "./Page.css";

export function JobsPage() {
	const [jobs, setJobs] = useState<Job[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const [showCreate, setShowCreate] = useState(false);
	const [jobName, setJobName] = useState("");
	const [jobParams, setJobParams] = useState("{}");
	const [creating, setCreating] = useState(false);
	const [createError, setCreateError] = useState<string | null>(null);

	const load = useCallback(async () => {
		try {
			setLoading(true);
			const result = await jobsApi.list();
			setJobs(result);
			setError(null);
		} catch (e) {
			setError(String(e));
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		load();
	}, [load]);

	const handleCreate = async (e: React.FormEvent) => {
		e.preventDefault();
		setCreateError(null);

		let parsed: unknown;
		try {
			parsed = JSON.parse(jobParams);
		} catch {
			setCreateError("Params は有効な JSON で入力してください");
			return;
		}

		const request: CreateOrUpdateJob = {
			name: jobName,
			params: parsed as Record<string, unknown>,
		};

		try {
			setCreating(true);
			await jobsApi.create(request);
			setJobName("");
			setJobParams("{}");
			setShowCreate(false);
			await load();
		} catch (e) {
			setCreateError(String(e));
		} finally {
			setCreating(false);
		}
	};

	return (
		<div className="page">
			<div className="breadcrumb">
				<Link to="/">Collections</Link>
				<span className="muted"> / </span>
				<span>Jobs</span>
			</div>

			<div className="page-header">
				<h1 className="page-title">Job 管理</h1>
				<div className="row-actions">
					<button
						className="btn btn-primary btn-sm"
						onClick={() => setShowCreate(!showCreate)}
					>
						+ Job 作成
					</button>
				</div>
			</div>

			{showCreate && (
				<form className="card form-card" onSubmit={handleCreate}>
					<h2 className="card-title">Job 作成</h2>
					<div className="form-group">
						<label className="form-label">Name</label>
						<input
							type="text"
							className="form-input"
							value={jobName}
							onChange={(e) => setJobName(e.target.value)}
							placeholder="job名を入力"
							required
						/>
					</div>
					<div className="form-group">
						<label className="form-label">Params (JSON)</label>
						<textarea
							className="form-textarea"
							value={jobParams}
							onChange={(e) => setJobParams(e.target.value)}
							rows={6}
							spellCheck={false}
							placeholder='{"key": "value"}'
						/>
					</div>
					{createError && <p className="error-text">{createError}</p>}
					<div className="form-actions">
						<button
							className="btn btn-primary"
							type="submit"
							disabled={creating}
						>
							{creating ? "作成中..." : "作成"}
						</button>
						<button
							className="btn btn-ghost"
							type="button"
							onClick={() => {
								setShowCreate(false);
								setJobName("");
								setJobParams("{}");
								setCreateError(null);
							}}
						>
							キャンセル
						</button>
					</div>
				</form>
			)}

			{loading && <p className="muted">読み込み中...</p>}
			{error && <p className="error-text">{error}</p>}

			{!loading && jobs.length === 0 && (
				<p className="muted">Job がありません</p>
			)}

			<div className="record-list">
				{jobs.map((job, index) => (
					<div key={index} className="card record-card">
						<div className="record-meta">
							<span className="record-id">{job.name}</span>
							<span
								className={`record-status ${
									job.status === "completed"
										? "status-success"
										: job.status === "failed"
											? "status-error"
											: "status-pending"
								}`}
							>
								{job.status}
							</span>
						</div>
						<pre className="code-block record-data">
							{JSON.stringify(job.params, null, 2)}
						</pre>
					</div>
				))}
			</div>
		</div>
	);
}

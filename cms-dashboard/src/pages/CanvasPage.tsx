// @specre 01KNM91QXA84F7HTBYSKM5TZMP
// @specre 01KNM91QXWGPMAEZZE0M7GCTB4
import { useCallback, useState } from "react";
import { canvasApi } from "../api/client";
import type { CanvasQueryResponse, Edge } from "../types";
import "./Page.css";

function formatDate(s: string | null) {
	if (!s) return "-";
	try {
		return new Date(s).toLocaleString("ja-JP");
	} catch {
		return s;
	}
}

export function CanvasPage() {
	// Edge listing
	const [edgeRecordId, setEdgeRecordId] = useState("");
	const [edges, setEdges] = useState<Edge[]>([]);
	const [edgesLoading, setEdgesLoading] = useState(false);
	const [edgesError, setEdgesError] = useState<string | null>(null);

	// Edge creation
	const [showCreateEdge, setShowCreateEdge] = useState(false);
	const [edgeFrom, setEdgeFrom] = useState("");
	const [edgeTo, setEdgeTo] = useState("");
	const [edgeLabel, setEdgeLabel] = useState("");
	const [createEdgeError, setCreateEdgeError] = useState<string | null>(null);
	const [creatingEdge, setCreatingEdge] = useState(false);

	// Subgraph query
	const [queryRecordId, setQueryRecordId] = useState("");
	const [queryDepth, setQueryDepth] = useState(1);
	const [queryFormat, setQueryFormat] = useState<"json-canvas" | "markdown">(
		"json-canvas",
	);
	const [queryResult, setQueryResult] = useState<CanvasQueryResponse | null>(
		null,
	);
	const [querying, setQuerying] = useState(false);
	const [queryError, setQueryError] = useState<string | null>(null);
	const [copied, setCopied] = useState(false);

	const loadEdges = useCallback(async () => {
		if (!edgeRecordId.trim()) return;
		try {
			setEdgesLoading(true);
			setEdgesError(null);
			const result = await canvasApi.listEdges(edgeRecordId.trim());
			setEdges(result);
		} catch (e) {
			setEdgesError(String(e));
		} finally {
			setEdgesLoading(false);
		}
	}, [edgeRecordId]);

	const handleCreateEdge = async (e: React.FormEvent) => {
		e.preventDefault();
		setCreateEdgeError(null);
		if (!edgeFrom.trim() || !edgeTo.trim() || !edgeLabel.trim()) {
			setCreateEdgeError("すべてのフィールドを入力してください");
			return;
		}
		try {
			setCreatingEdge(true);
			await canvasApi.createEdge({
				from: edgeFrom.trim(),
				to: edgeTo.trim(),
				label: edgeLabel.trim(),
			});
			setEdgeFrom("");
			setEdgeTo("");
			setEdgeLabel("");
			setShowCreateEdge(false);
			if (edgeRecordId.trim()) await loadEdges();
		} catch (e) {
			setCreateEdgeError(String(e));
		} finally {
			setCreatingEdge(false);
		}
	};

	const handleDeleteEdge = async (edgeId: string) => {
		if (!confirm("このエッジを削除しますか？")) return;
		try {
			await canvasApi.deleteEdge(edgeId);
			setEdges((prev) => prev.filter((e) => e.id !== edgeId));
		} catch (e) {
			setEdgesError(String(e));
		}
	};

	const handleQuery = async (e: React.FormEvent) => {
		e.preventDefault();
		setQueryError(null);
		setQueryResult(null);
		if (!queryRecordId.trim()) {
			setQueryError("レコード ID を入力してください");
			return;
		}
		try {
			setQuerying(true);
			const result = await canvasApi.query({
				record_id: queryRecordId.trim(),
				depth: queryDepth,
				format: queryFormat,
			});
			setQueryResult(result);
		} catch (e) {
			setQueryError(String(e));
		} finally {
			setQuerying(false);
		}
	};

	const handleCopy = async () => {
		if (!queryResult) return;
		let text: string;
		if (queryResult.format === "json-canvas" && queryResult.json_canvas) {
			text = JSON.stringify(queryResult.json_canvas, null, 2);
		} else if (queryResult.format === "markdown" && queryResult.markdown) {
			text = queryResult.markdown;
		} else {
			return;
		}
		await navigator.clipboard.writeText(text);
		setCopied(true);
		setTimeout(() => setCopied(false), 2000);
	};

	return (
		<div className="page">
			<div className="page-header">
				<h1 className="page-title">Canvas</h1>
			</div>

			{/* Edge Management Section */}
			<div className="card">
				<h2 className="card-title">エッジ管理</h2>

				<div className="edge-search-row">
					<div className="form-group" style={{ marginBottom: 0, flex: 1 }}>
						<label className="form-label">レコード ID（table:id 形式）</label>
						<input
							className="form-input"
							value={edgeRecordId}
							onChange={(e) => setEdgeRecordId(e.target.value)}
							placeholder="例: characters:abc123"
						/>
					</div>
					<button
						className="btn btn-primary"
						onClick={loadEdges}
						disabled={edgesLoading || !edgeRecordId.trim()}
						style={{ alignSelf: "flex-end" }}
					>
						{edgesLoading ? "読込中..." : "エッジ取得"}
					</button>
					<button
						className="btn btn-ghost"
						onClick={() => setShowCreateEdge(!showCreateEdge)}
						style={{ alignSelf: "flex-end" }}
					>
						{showCreateEdge ? "キャンセル" : "+ エッジ作成"}
					</button>
				</div>

				{edgesError && <p className="error-text">{edgesError}</p>}

				{showCreateEdge && (
					<form onSubmit={handleCreateEdge} className="edge-create-form">
						<div className="form-group">
							<label className="form-label">From（送信元レコード ID）</label>
							<input
								className="form-input"
								value={edgeFrom}
								onChange={(e) => setEdgeFrom(e.target.value)}
								placeholder="例: characters:abc123"
							/>
						</div>
						<div className="form-group">
							<label className="form-label">To（送信先レコード ID）</label>
							<input
								className="form-input"
								value={edgeTo}
								onChange={(e) => setEdgeTo(e.target.value)}
								placeholder="例: organizations:def456"
							/>
						</div>
						<div className="form-group">
							<label className="form-label">Label</label>
							<input
								className="form-input"
								value={edgeLabel}
								onChange={(e) => setEdgeLabel(e.target.value)}
								placeholder="例: belongs_to"
							/>
						</div>
						{createEdgeError && <p className="error-text">{createEdgeError}</p>}
						<div className="form-actions">
							<button
								className="btn btn-primary"
								type="submit"
								disabled={creatingEdge}
							>
								{creatingEdge ? "作成中..." : "作成"}
							</button>
						</div>
					</form>
				)}

				{edges.length > 0 && (
					<div className="edge-table-wrap">
						<table className="edge-table">
							<thead>
								<tr>
									<th>Label</th>
									<th>From</th>
									<th>To</th>
									<th>作成日時</th>
									<th></th>
								</tr>
							</thead>
							<tbody>
								{edges.map((edge) => (
									<tr key={edge.id}>
										<td>
											<span className="edge-label-badge">{edge.label}</span>
										</td>
										<td className="mono-text">{edge.from}</td>
										<td className="mono-text">{edge.to}</td>
										<td className="muted">{formatDate(edge.created_at)}</td>
										<td>
											<button
												className="btn btn-danger btn-sm"
												onClick={() => handleDeleteEdge(edge.id)}
											>
												削除
											</button>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				)}

				{!edgesLoading &&
					edges.length === 0 &&
					edgeRecordId.trim() &&
					!edgesError && <p className="muted">エッジが見つかりません</p>}
			</div>

			{/* Subgraph Query Section */}
			<div className="card">
				<h2 className="card-title">部分グラフクエリ</h2>

				<form onSubmit={handleQuery} className="query-form">
					<div className="query-form-row">
						<div className="form-group" style={{ flex: 2 }}>
							<label className="form-label">起点レコード ID</label>
							<input
								className="form-input"
								value={queryRecordId}
								onChange={(e) => setQueryRecordId(e.target.value)}
								placeholder="例: characters:abc123"
							/>
						</div>
						<div className="form-group" style={{ flex: 0.5 }}>
							<label className="form-label">深度</label>
							<input
								className="form-input"
								type="number"
								min={1}
								max={10}
								value={queryDepth}
								onChange={(e) => setQueryDepth(Number(e.target.value))}
							/>
						</div>
						<div className="form-group" style={{ flex: 1 }}>
							<label className="form-label">形式</label>
							<select
								className="form-select"
								value={queryFormat}
								onChange={(e) =>
									setQueryFormat(e.target.value as "json-canvas" | "markdown")
								}
							>
								<option value="json-canvas">JSON Canvas</option>
								<option value="markdown">Markdown</option>
							</select>
						</div>
						<button
							className="btn btn-primary"
							type="submit"
							disabled={querying || !queryRecordId.trim()}
							style={{ alignSelf: "flex-end" }}
						>
							{querying ? "クエリ中..." : "クエリ実行"}
						</button>
					</div>
				</form>

				{queryError && <p className="error-text">{queryError}</p>}

				{queryResult && (
					<div className="query-result">
						<div className="query-result-header">
							<span className="query-result-meta">
								{queryResult.format === "json-canvas" && queryResult.json_canvas
									? `${queryResult.json_canvas.nodes.length} ノード / ${queryResult.json_canvas.edges.length} エッジ`
									: "Markdown"}
							</span>
							<button className="btn btn-ghost btn-sm" onClick={handleCopy}>
								{copied ? "コピー済み!" : "コピー"}
							</button>
						</div>
						<pre className="code-block query-result-code">
							{queryResult.format === "json-canvas" && queryResult.json_canvas
								? JSON.stringify(queryResult.json_canvas, null, 2)
								: (queryResult.markdown ?? "")}
						</pre>
					</div>
				)}
			</div>
		</div>
	);
}

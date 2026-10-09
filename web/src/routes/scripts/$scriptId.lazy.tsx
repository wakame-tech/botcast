import { JsonSchemaForm } from "@/components/JsonSchemaForm";
import { Button } from "@/components/ui/button";
import { $cms, recordToScript } from "@/lib/cms_client";
import { Link, createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/scripts/$scriptId")({
	component: Script,
});

/** 試し実行で渡す context (worker の GenerateEpisode と同じ形のサンプル値) */
const sampleContext = (args: Record<string, unknown>) => ({
	podcast: {
		id: "sample-podcast",
		title: "サンプル番組",
		icon: "",
		description: "試し実行用の番組",
		user_id: "sample-user",
	},
	previous_episode: {
		id: "sample-episode",
		title: "第1回",
		podcast_id: "sample-podcast",
		description: "",
		sections: [],
		user_id: "sample-user",
	},
	mails: [
		{
			id: "sample-mail-1",
			podcast_id: "sample-podcast",
			episode_id: "sample-episode",
			radio_name: "ラジオネーム テスト",
			body: "いつも楽しく聴いています。",
			user_id: "sample-user",
		},
		{
			id: "sample-mail-2",
			podcast_id: "sample-podcast",
			episode_id: "sample-episode",
			radio_name: "匿名希望",
			body: "次回のテーマを教えてください。",
			user_id: "sample-user",
		},
	],
	arguments: args,
});

export function Script() {
	const navigate = useNavigate();
	const { scriptId } = Route.useParams();
	const { data: scriptRecord } = $cms.useQuery(
		"get",
		"/records/{collectionId}/{recordId}",
		{ params: { path: { collectionId: "scripts", recordId: scriptId } } },
	);
	const [parameters, setParameters] = useState<Record<string, unknown>>({});
	const deleteScript = $cms.useMutation(
		"delete",
		"/records/{collectionId}/{recordId}",
	);
	const executeScript = $cms.useMutation("post", "/scripts");
	const [executeError, setExecuteError] = useState<string>();

	if (!scriptRecord) {
		return null;
	}

	const handleDelete = async () => {
		await deleteScript.mutateAsync({
			params: { path: { collectionId: "scripts", recordId: scriptId } },
		});
		navigate({ to: "/scripts" });
	};

	const script = recordToScript(scriptRecord);

	const handleExecute = async (args: Record<string, unknown>) => {
		const context = sampleContext(args);
		setExecuteError(undefined);
		try {
			await executeScript.mutateAsync({
				body: {
					language: "nodejs",
					code: `const context = ${JSON.stringify(context)};\n${script.template}`,
				},
			});
		} catch (e) {
			setExecuteError(String(e));
		}
	};

	return (
		<>
			<h1>{script.title}</h1>

			<p>{script.description}</p>

			<Link to="/scripts/$scriptId/edit" params={{ scriptId }}>
				<Button>edit</Button>
			</Link>
			<Button onClick={handleDelete}>delete</Button>

			<pre className="p-2 text-sm bg-gray-1">
				<code>{script.template}</code>
			</pre>

			<h2 className="text-lg font-bold">試し実行</h2>
			{Object.keys(script.arguments).length !== 0 ? (
				<JsonSchemaForm
					schema={script.arguments}
					formData={parameters}
					onChange={(e) => setParameters(e)}
					onSubmit={handleExecute}
				/>
			) : (
				<Button onClick={() => handleExecute({})}>実行</Button>
			)}

			{executeScript.isPending && <p>実行中...</p>}
			{executeError && <p className="text-red-600">{executeError}</p>}
			{executeScript.data && (
				<>
					<h3 className="font-bold">標準出力</h3>
					<pre className="p-2 text-sm bg-gray-1 whitespace-pre-wrap">
						<code>{executeScript.data.data.stdout}</code>
					</pre>
					{executeScript.data.data.error && (
						<>
							<h3 className="font-bold">エラー</h3>
							<pre className="p-2 text-sm bg-gray-1 text-red-600 whitespace-pre-wrap">
								<code>{executeScript.data.data.error}</code>
							</pre>
						</>
					)}
				</>
			)}
		</>
	);
}

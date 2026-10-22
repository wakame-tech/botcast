import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { CmsAudio, CmsPlayButton } from "@/components/voice/VoiceAudio";
import { VoiceIcon } from "@/components/voice/VoiceIcon";
import {
	$cms,
	CMS_TENANT,
	type Cast,
	type Voice,
	recordToVoice,
} from "@/lib/cms_client";
import { Link } from "@tanstack/react-router";
import { useState } from "react";

/**
 * 声の一覧 (slug → 声)。voices は所有者しか読めないので、自分の声だけが返る。
 * `enabled` が false なら取得しない
 */
export const useVoices = (enabled = true) => {
	const { data: records, isPending } = $cms.useQuery(
		"get",
		"/t/{tenantId}/records/{collectionId}",
		{
			params: { path: { tenantId: CMS_TENANT, collectionId: "voices" } },
		},
		{ enabled },
	);
	const voices = (records ?? []).map(recordToVoice);
	return {
		voices: new Map<string, Voice>(voices.map((v) => [v.slug, v])),
		isPending: enabled && isPending,
	};
};

interface Row {
	key: number;
	alias: string;
	slug: string;
}

let nextRowKey = 0;

interface CastFormProps {
	value: Cast;
	onSubmit: (cast: Cast) => void;
}

/** 番組の配役 (台本の speaker に書く別名 → 声) の編集 */
export function CastForm({ value, onSubmit }: CastFormProps) {
	const { voices } = useVoices();
	const [rows, setRows] = useState<Row[]>(
		Object.entries(value).map(([alias, slug]) => ({
			key: nextRowKey++,
			alias,
			slug,
		})),
	);
	const aliases = rows.map((r) => r.alias.trim()).filter((a) => a !== "");
	const duplicated = aliases.length !== new Set(aliases).size;
	const missingAlias = rows.some((r) => r.alias.trim() === "" && r.slug !== "");
	// 数字だけの別名は旧形式の speaker (VOICEVOX のスタイル ID) と取り違えるので使えない
	const numericAlias = aliases.some((a) => /^[0-9]+$/.test(a));

	const update = (key: number, patch: Partial<Row>) =>
		setRows(rows.map((r) => (r.key === key ? { ...r, ...patch } : r)));

	return (
		<form
			className="flex flex-col gap-2 py-4"
			onSubmit={(e) => {
				e.preventDefault();
				onSubmit(
					Object.fromEntries(
						rows
							.filter((r) => r.alias.trim() !== "" && r.slug !== "")
							.map((r) => [r.alias.trim(), r.slug]),
					),
				);
			}}
		>
			<h2 className="text-lg font-bold">配役</h2>
			<p className="text-sm text-gray-500">
				台本の speaker には別名を書きます。声は
				<Link to="/voices">声の一覧</Link>で作成できます。
			</p>
			{rows.map((row) => (
				<div key={row.key} className="flex flex-wrap gap-2 items-center">
					<Input
						className="w-40"
						placeholder="別名 (例: ホスト)"
						value={row.alias}
						onChange={(e) => update(row.key, { alias: e.target.value })}
					/>
					<select
						className="border rounded px-2 py-1"
						value={row.slug}
						onChange={(e) => update(row.key, { slug: e.target.value })}
						required
					>
						<option value="">声を選択</option>
						{[...voices.values()].map((v) => (
							<option key={v.slug} value={v.slug}>
								{v.name} ({v.slug})
							</option>
						))}
						{row.slug !== "" && !voices.has(row.slug) && (
							<option value={row.slug}>{row.slug} (見つかりません)</option>
						)}
					</select>
					<CmsAudio file={voices.get(row.slug)?.sample} />
					<Button
						type="button"
						onClick={() => setRows(rows.filter((r) => r.key !== row.key))}
					>
						削除
					</Button>
				</div>
			))}
			<div>
				<Button
					type="button"
					onClick={() =>
						setRows([...rows, { key: nextRowKey++, alias: "", slug: "" }])
					}
				>
					配役を追加
				</Button>
			</div>
			{duplicated && (
				<p className="text-sm text-red-500">別名が重複しています</p>
			)}
			{missingAlias && (
				<p className="text-sm text-red-500">別名が空の配役があります</p>
			)}
			{numericAlias && (
				<p className="text-sm text-red-500">
					数字だけの別名は使えません (旧形式の speaker と区別するため)
				</p>
			)}
			<Button
				type="submit"
				disabled={duplicated || missingAlias || numericAlias}
			>
				配役を保存
			</Button>
		</form>
	);
}

interface CastListProps {
	cast: Cast;
	/**
	 * 声を読めるか (番組の持ち主か)。voices は所有者しか読めないので、
	 * 持ち主以外には声を引かず別名と slug だけを出す
	 */
	canReadVoices: boolean;
}

/** 番組の配役の一覧 (アイコン・声の画面へのリンク・試聴付き) */
export function CastList({ cast, canReadVoices }: CastListProps) {
	const { voices, isPending } = useVoices(canReadVoices);
	const entries = Object.entries(cast);
	if (entries.length === 0) {
		return <p className="text-sm text-gray-500">配役はありません</p>;
	}
	return (
		<ul className="flex flex-col gap-2 py-2">
			{entries.map(([alias, slug]) => {
				const voice = voices.get(slug);
				return (
					<li key={alias} className="flex flex-wrap gap-2 items-center">
						<VoiceIcon file={voice?.icon} />
						<span className="font-bold">{alias}</span>
						{voice ? (
							<>
								<Link to="/voices/$voiceId" params={{ voiceId: voice.id }}>
									{voice.name}
								</Link>
								<CmsPlayButton file={voice.sample} />
							</>
						) : canReadVoices && !isPending ? (
							<span className="text-red-500">{slug} (見つかりません)</span>
						) : (
							<span className="text-sm text-gray-500">{slug}</span>
						)}
					</li>
				);
			})}
		</ul>
	);
}

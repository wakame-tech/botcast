import { fetchCmsFile } from "@/lib/cms_client";
import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";

/** CMS に保存されたファイルを取得し、再生などに使える Object URL を返す */
export const useCmsFileUrl = (path: string | null | undefined) => {
	const { data } = useQuery({
		queryKey: ["cms-file", path],
		queryFn: () => fetchCmsFile(path ?? ""),
		enabled: !!path,
		staleTime: Number.POSITIVE_INFINITY,
	});
	const [url, setUrl] = useState<string | null>(null);
	// Object URL は effect 内で作って破棄する (StrictMode の再実行で無効な URL を使い続けないように)
	useEffect(() => {
		if (!data) {
			setUrl(null);
			return;
		}
		const objectUrl = URL.createObjectURL(data);
		setUrl(objectUrl);
		return () => URL.revokeObjectURL(objectUrl);
	}, [data]);
	return url;
};

import { supabase } from "@/supabase";
import type { Session } from "@supabase/supabase-js";
import { useEffect, useState } from "react";

export const useSession = () => {
	const [session, setSession] = useState<Session | null>(null);

	useEffect(() => {
		// 初期セッションを取得
		const getInitialSession = async () => {
			const { data: { session } } = await supabase.auth.getSession();
			setSession(session);
		};

		getInitialSession();

		// 認証状態の変更を監視
		const { data: { subscription } } = supabase.auth.onAuthStateChange((event, session) => {
			setSession(session);
		});

		// クリーンアップ関数でリスナーを削除
		return () => subscription.unsubscribe();
	}, []);

	return {
		session,
		setSession,
	};
};

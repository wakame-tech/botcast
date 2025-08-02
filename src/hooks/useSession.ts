import { supabase } from "@/supabase";
import type { Session } from "@supabase/supabase-js";
import { useEffect, useState } from "react";

export const useSession = () => {
	const [session, setSession] = useState<Session | null>(null);

	useEffect(() => {
		supabase.auth.onAuthStateChange((event, session) => {
			if (event === "SIGNED_IN" && session !== null) {
				supabase.auth.setSession(session);
			}
		});

		supabase.auth.getSession().then(({ data: { session } }) => {
			session?.access_token;
			setSession(session);
		});
	}, []);

	return {
		session,
		setSession,
	};
};

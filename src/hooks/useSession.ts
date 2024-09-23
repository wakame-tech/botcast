import { useEffect, useState } from "react";
import { supabase } from "../supabase";
import type { Session } from "@supabase/supabase-js";

export const useSession = () => {
    const [session, setSession] = useState<Session | null>(null);

    useEffect(() => {
        supabase.auth.getSession().then(({ data: { session } }) => {
            setSession(session);
        });
    }, []);

    return {
        session,
        setSession,
    };
};

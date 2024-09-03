import { useEffect, useState } from "react";
import { supabase } from "../supabase";
import { Session } from "@supabase/supabase-js";

export const useSession = () => {
    const [session, setSession] = useState<Session | null>(null);

    const signOut = async () => {
        await supabase.auth.signOut();
    };

    useEffect(() => {
        supabase.auth.getSession().then(({ data: { session } }) => {
            setSession(session);
        });

        const { data: { subscription } } = supabase.auth.onAuthStateChange(
            (_event, session) => {
                setSession(session);
            },
        );

        return () => subscription.unsubscribe();
    }, []);

    return {
        session,
        signOut,
    };
};

import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { supabase } from "../supabase";
import { Auth } from "@supabase/auth-ui-react";
import { ThemeSupa } from "@supabase/auth-ui-shared";
import { useSession } from "../hooks/useSession";
import { useEffect } from "react";

export const Route = createLazyFileRoute("/signin")({
	component: Signin,
});

function Signin() {
	const { session } = useSession();
	const navigate = useNavigate();

	useEffect(() => {
		if (session) {
			navigate({ to: "/" });
		}
	}, [session, navigate]);

	return <Auth supabaseClient={supabase} appearance={{ theme: ThemeSupa }} />;
}

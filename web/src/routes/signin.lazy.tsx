import { supabase } from "@/supabase";
import { Auth } from "@supabase/auth-ui-react";
import { ThemeSupa } from "@supabase/auth-ui-shared";
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/signin")({
	component: Signin,
});

function Signin() {
	return (
		<Auth
			supabaseClient={supabase}
			appearance={{ theme: ThemeSupa }}
			providers={[]}
		/>
	);
}

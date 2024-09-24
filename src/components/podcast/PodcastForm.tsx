import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { trpc } from "@/trpc.ts";
import { zodResolver } from "@hookform/resolvers/zod";
import type { Podcast } from "@prisma/client";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface PodcastFormProps {
	onCreate: (podcast: Podcast) => void;
}

const formSchema = z.object({
	title: z.string(),
});

export function PodcastForm(props: PodcastFormProps) {
	const newPodcast = trpc.newPodcast.useMutation();

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			title: "",
		},
	});

	const onSubmit = async (values: z.infer<typeof formSchema>) => {
		const { podcast } = await newPodcast.mutateAsync({ ...values, icon: "🎧" });
		props.onCreate(podcast);
	};

	return (
		<Form {...form}>
			<form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
				<FormField
					control={form.control}
					name="title"
					render={({ field }) => (
						<FormItem>
							<FormLabel>タイトル</FormLabel>
							<FormControl>
								<Input placeholder="title" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<Button type="submit">作成</Button>
			</form>
		</Form>
	);
}

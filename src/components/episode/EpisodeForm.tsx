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
import type { Episode } from "@prisma/client";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface EpisodeFormProps {
	podcastId: string;
	onCreate: (episode: Episode) => void;
}

const formSchema = z.object({
	title: z.string(),
	url: z.string(),
});

export function EpisodeForm(props: EpisodeFormProps) {
	const newEpisode = trpc.newEpisode.useMutation();

	const form = useForm<z.infer<typeof formSchema>>({
		resolver: zodResolver(formSchema),
		defaultValues: {
			title: "",
			url: "",
		},
	});

	const onSubmit = async (values: z.infer<typeof formSchema>) => {
		const { episode } = await newEpisode.mutateAsync({
			...values,
			podcastId: props.podcastId,
		});
		props.onCreate(episode);
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
				<FormField
					control={form.control}
					name="url"
					render={({ field }) => (
						<FormItem>
							<FormLabel>URL</FormLabel>
							<FormControl>
								<Input placeholder="URL" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>
				<Button type="submit">作成</Button>
			</form>
		</Form>
	);
}

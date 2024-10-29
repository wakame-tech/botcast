import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
	FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { zodResolver } from "@hookform/resolvers/zod";
import type { PropsWithChildren } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

interface PodcastFormProps {
	values?: PodcastEditFormValues;
	onSubmit: (values: PodcastEditFormValues) => void;
}

const WEEK_DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

const weekDays = z.enum(WEEK_DAYS);

const podcastEditFormSchema = z.object({
	title: z.string().min(1).max(50),
	weekDay: weekDays,
	hour: z.coerce.number().int().min(0).max(23),
});

export type PodcastEditFormValues = z.infer<typeof podcastEditFormSchema>;

export function PodcastForm(props: PropsWithChildren<PodcastFormProps>) {
	const form = useForm<z.infer<typeof podcastEditFormSchema>>({
		resolver: zodResolver(podcastEditFormSchema),
		defaultValues: props.values ?? {
			title: "新しいポッドキャスト",
			weekDay: "Mon",
			hour: 0,
		},
	});

	const onSubmit = async (values: PodcastEditFormValues) => {
		form.reset();
		props.onSubmit(values);
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
							<FormMessage />
						</FormItem>
					)}
				/>
				<div className="grid grid-cols-2 gap-4">
					<FormField
						control={form.control}
						name="weekDay"
						render={({ field }) => (
							<FormItem>
								<FormLabel>曜日</FormLabel>
								<FormControl>
									<Select defaultValue={field.value}>
										<SelectTrigger>
											<SelectValue />
										</SelectTrigger>
										<SelectContent>
											{WEEK_DAYS.map((d) => (
												<SelectItem key={d} value={d}>
													{d}
												</SelectItem>
											))}
										</SelectContent>
									</Select>
								</FormControl>
								<FormMessage />
							</FormItem>
						)}
					/>

					<FormField
						control={form.control}
						name="hour"
						render={({ field }) => (
							<FormItem>
								<FormLabel>時間</FormLabel>
								<FormControl>
									<Input type="number" placeholder="hour" {...field} />
								</FormControl>
								<FormMessage />
							</FormItem>
						)}
					/>
				</div>

				{props.children}

				<Button disabled={!form.formState.isValid} type="submit">
					作成
				</Button>
			</form>
		</Form>
	);
}

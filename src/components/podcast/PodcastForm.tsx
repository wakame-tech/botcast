import { Button } from "@/components/ui/button";
import {
	Form,
	FormControl,
	FormField,
	FormItem,
	FormLabel,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { z } from "zod";
import { MANUSCRIPT_TEMPLATE } from "../script/ScriptForm";

interface PodcastFormProps {
	values?: PodcastEditFormValues;
	onSubmit: (values: PodcastEditFormValues) => void;
}

const WEEK_DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] as const;

const weekDays = z.enum(WEEK_DAYS);

const podcastEditFormSchema = z.object({
	title: z.string(),
	template: z.string(),
	weekDay: weekDays,
	hour: z.number().int().min(0).max(23),
});

export type PodcastEditFormValues = z.infer<typeof podcastEditFormSchema>;

export function PodcastForm(props: PodcastFormProps) {
	const form = useForm<z.infer<typeof podcastEditFormSchema>>({
		resolver: zodResolver(podcastEditFormSchema),
		defaultValues: props.values ?? {
			title: "新しいポッドキャスト",
			template: MANUSCRIPT_TEMPLATE,
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
									<Select>
										<SelectTrigger>
											<SelectValue placeholder="曜日" {...field} />
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
									<Input placeholder="hour" {...field} />
								</FormControl>
							</FormItem>
						)}
					/>
				</div>
				<FormField
					control={form.control}
					name="template"
					render={({ field }) => (
						<FormItem>
							<FormLabel>テンプレートスクリプト</FormLabel>
							<FormControl>
								<Textarea rows={10} placeholder="template" {...field} />
							</FormControl>
						</FormItem>
					)}
				/>

				<Button type="submit">作成</Button>
			</form>
		</Form>
	);
}

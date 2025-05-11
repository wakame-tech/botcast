import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { Podcast, User } from "@/lib/api_client";
import { Link } from "@tanstack/react-router";
import { UserIcon } from "../user/UserIcon";

interface PodcastListProps {
	podcasts: (Podcast & { user: User | null })[];
}

function PodcastList(props: PodcastListProps) {
	return (
		<>
			{props.podcasts.map((podcast) => (
				<div key={podcast.id} className="p-2">
					<PodcastListItem podcast={podcast} />
				</div>
			))}
		</>
	);
}

interface PodcastListItemProps {
	podcast: Podcast & { user: User | null };
}

function PodcastListItem(props: PodcastListItemProps) {
	return (
		<>
			<Card>
				<CardHeader>
					<CardTitle>
						<div className="pb-2">
							<UserIcon
								size="1.5rem"
								userId={props.podcast.user?.id ?? ""}
								label={props.podcast.user?.name ?? ""}
							/>
						</div>
						<div className="flex-inline items-center gap-2">
							<span className="bg-teal-300 w-16 h-16 rounded-xl flex items-center justify-center">
								{props.podcast.icon}
							</span>
							<Link
								to="/podcasts/$podcastId"
								params={{ podcastId: props.podcast.id }}
								className="no-underline"
							>
								<span className="pl-2 text-xl font-bold">
									{props.podcast.title}
								</span>
							</Link>
						</div>
					</CardTitle>
				</CardHeader>
				<CardContent>{props.podcast.description}</CardContent>
			</Card>
		</>
	);
}

const components = {
	List: PodcastList,
	Item: PodcastListItem,
};

export default components;

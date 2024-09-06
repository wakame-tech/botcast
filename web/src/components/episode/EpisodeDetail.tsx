import { Episode, User } from "@prisma/client"

interface EpisodeDetailProps {
    episode: Episode
    user: User | null
}

export function EpisodeDetail({ episode, user }: EpisodeDetailProps) {
    return (
        <>
            <p className="text-xl">{episode.title}<span className="text-gray text-sm">{user ? `@${user.name}` : ''}</span></p>

            {episode.audio_url && <audio src={episode.audio_url} controls />}
            {episode.content && <p className="whitespace-pre-wrap">{episode.content}</p>}
        </>
    )
}

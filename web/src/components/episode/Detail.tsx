import { Episode } from "@prisma/client"

interface EpisodeDetailProps {
    episode: Episode
}

export function EpisodeDetail({ episode }: EpisodeDetailProps) {
    return (
        <>
            <h3>{episode.title}</h3>
            {episode.audio_url && <audio src={episode.audio_url} controls />}
        </>
    )
}

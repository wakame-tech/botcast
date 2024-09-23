import type { User } from "@prisma/client"
import type { Line } from 'srt-parser-2'

interface EpisodeDetailProps {
    title: string
    audioUrl: string | null
    lines: Line[]
    user: User | null
}

export function EpisodeDetail({ title, audioUrl, lines, user }: EpisodeDetailProps) {
    return (
        <>
            <p className="text-xl">{title}<span className="text-gray text-sm">{user ? `@${user.name}` : ''}</span></p>

            {
                // biome-ignore lint:
                audioUrl && <audio src={audioUrl} controls />
            }
            {lines.map(line => <div className="hover:bg-gray-200" key={`t${line.startSeconds}`}>
                <p className="text-lg py-2">
                    {line.text}
                </p>
            </div>)
            }
        </>
    )
}

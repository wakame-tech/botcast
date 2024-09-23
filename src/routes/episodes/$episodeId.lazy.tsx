import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../../trpc.ts";
import Parser from 'srt-parser-2'
import type { Line } from 'srt-parser-2'
import { EpisodeDetail } from '../../components/episode/EpisodeDetail.tsx';
import { useEffect, useState } from 'react';

export const Route = createLazyFileRoute('/episodes/$episodeId')({
  component: Episode,
})

const fetchSrt = async (url: string): Promise<Line[]> => {
  const res = await fetch(url)
  if (!res.ok) {
    throw new Error(`Failed to fetch ${url}: ${res.status}`)
  }
  const srt = await res.text()
  const parser = new Parser();
  return parser.fromSrt(srt);
}

function Episode() {
  const { episodeId } = Route.useParams()
  const episodeQuery = trpc.episode.useQuery({ id: episodeId });
  const episode = episodeQuery.data?.episode
  const [lines, setLines] = useState<Line[]>([])

  useEffect(() => {
    (async () => {
      if (!episode || !episode?.script_url) {
        return
      }
      const lines = await fetchSrt(episode.script_url)
      setLines(lines);
    })()
  }, [episode])

  if (!episode) {
    return <div>not found</div>
  }

  return (
    <div className='p-2'>
      <EpisodeDetail title={episode.title} audioUrl={episode.audio_url} lines={lines} user={episode.user} />
    </div>
  )
}

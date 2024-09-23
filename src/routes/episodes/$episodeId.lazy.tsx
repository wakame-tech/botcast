import { createLazyFileRoute } from '@tanstack/react-router'
import { trpc } from "../../trpc.ts";
import { EpisodeDetail } from '../../components/episode/EpisodeDetail.tsx';

export const Route = createLazyFileRoute('/episodes/$episodeId')({
  component: Episode,
})

function Episode() {
  const { episodeId } = Route.useParams()
  const episodeQuery = trpc.episode.useQuery({ id: episodeId });
  const episode = episodeQuery.data?.episode

  if (!episode) {
    return <div>not found</div>
  }

  return (
    <div className='p-2'>
      <EpisodeDetail episode={episode} user={episode.user} />
    </div>
  )
}
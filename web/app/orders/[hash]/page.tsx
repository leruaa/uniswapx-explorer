import { HydrationBoundary, QueryClient, dehydrate } from '@tanstack/react-query';
import Order from './order';

export default function Page({ params }: { params: { hash: string } }) {
  const queryClient = new QueryClient()

  return (
    <HydrationBoundary state={dehydrate(queryClient)}>
      <Order hash={params.hash} />
    </HydrationBoundary>
  )
}


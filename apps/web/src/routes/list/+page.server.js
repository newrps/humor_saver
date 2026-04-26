import { listArticles } from '$lib/api.js';

export async function load({ fetch, url }) {
  const q = url.searchParams.get('q') || '';
  const sourceId = url.searchParams.get('source_id') || '';
  const data = await listArticles(fetch, {
    limit: 30,
    q: q || undefined,
    source_id: sourceId || undefined
  });
  return { articles: data.items, q, sourceId };
}

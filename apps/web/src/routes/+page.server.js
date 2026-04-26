import { listArticles } from '$lib/api.js';

export async function load({ fetch, url }) {
  const limit = 20;
  const offset = Number(url.searchParams.get('offset') || '0');
  const data = await listArticles(fetch, { limit, offset });
  return { articles: data.items, offset, limit };
}

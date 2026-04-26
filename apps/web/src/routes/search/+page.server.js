import { semanticSearch } from '$lib/api.js';

export async function load({ fetch, url }) {
  const q = url.searchParams.get('q') || '';
  if (!q.trim()) return { q, results: [] };
  try {
    const data = await semanticSearch(fetch, q, 30);
    return { q, results: data.items };
  } catch (e) {
    return { q, results: [], error: String(e) };
  }
}

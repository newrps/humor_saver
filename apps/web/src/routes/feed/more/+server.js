// 무한 스크롤용 페이지네이션 endpoint (JSON 반환)
import { json } from '@sveltejs/kit';
import { listArticles } from '$lib/api.js';

export async function GET({ fetch, url }) {
  const limit = Number(url.searchParams.get('limit') || '20');
  const offset = Number(url.searchParams.get('offset') || '0');
  const data = await listArticles(fetch, { limit, offset });
  return json({ items: data.items, offset, limit });
}

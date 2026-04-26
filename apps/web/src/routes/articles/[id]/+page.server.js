import { error } from '@sveltejs/kit';
import { getArticle } from '$lib/api.js';

export async function load({ fetch, params }) {
  try {
    const article = await getArticle(fetch, params.id);
    return { article };
  } catch (e) {
    throw error(404, '기사를 찾을 수 없습니다');
  }
}

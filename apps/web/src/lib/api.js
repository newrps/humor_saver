// SvelteKit SSR 에서만 호출 (load 함수). 브라우저 직접 호출 X.
// → API 컨테이너는 외부 노출 없이 docker 내부 네트워크에서만 접근

const API = process.env.INTERNAL_API_URL || 'http://api:8080';

async function jsonFetch(url, fetchFn) {
  const r = await fetchFn(url);
  if (!r.ok) throw new Error(`API ${r.status}: ${url}`);
  return r.json();
}

export async function listArticles(fetchFn, { limit = 20, offset = 0, source_id, q } = {}) {
  const u = new URL('/articles', API);
  u.searchParams.set('limit', limit);
  u.searchParams.set('offset', offset);
  if (source_id) u.searchParams.set('source_id', source_id);
  if (q) u.searchParams.set('q', q);
  return jsonFetch(u, fetchFn);
}

export async function semanticSearch(fetchFn, q, limit = 20) {
  const u = new URL('/search', API);
  u.searchParams.set('q', q);
  u.searchParams.set('limit', limit);
  return jsonFetch(u, fetchFn);
}

export async function listSources(fetchFn) {
  return jsonFetch(new URL('/sources', API), fetchFn);
}

export async function trends(fetchFn, days = 7, limit = 30) {
  const u = new URL('/trends/keywords', API);
  u.searchParams.set('days', days);
  u.searchParams.set('limit', limit);
  return jsonFetch(u, fetchFn);
}

export async function getArticle(fetchFn, id) {
  return jsonFetch(new URL(`/articles/${id}`, API), fetchFn);
}

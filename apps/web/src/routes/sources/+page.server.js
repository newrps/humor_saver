import { error } from '@sveltejs/kit';
import { listSources } from '$lib/api.js';

export async function load({ fetch, locals }) {
  // hooks.server.js 에서 이미 차단되지만 안전장치
  if (!locals.isHomeIp) throw error(404, 'Not Found');
  const sources = await listSources(fetch);
  return { sources };
}

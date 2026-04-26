import { trends } from '$lib/api.js';

export async function load({ fetch, url }) {
  const days = Number(url.searchParams.get('days') || '7');
  const data = await trends(fetch, days, 50);
  return { keywords: data.items, days };
}

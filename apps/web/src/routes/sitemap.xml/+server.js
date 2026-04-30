import { listArticles } from '$lib/api.js';

const ORIGIN = 'https://fun.zam.kr';
const STATIC_PAGES = [
  { p: '/', cf: 'hourly', pr: '1.0' },
  { p: '/list', cf: 'hourly', pr: '0.9' },
  { p: '/search', cf: 'daily', pr: '0.8' },
  { p: '/sources', cf: 'weekly', pr: '0.5' }
];

function esc(s) {
  return String(s ?? '').replace(/[<>&'"]/g, (c) =>
    ({'<':'&lt;','>':'&gt;','&':'&amp;','\'':'&apos;','"':'&quot;'})[c]
  );
}

export async function GET({ fetch }) {
  let items = [];
  try {
    const data = await listArticles(fetch, { limit: 1000 });
    items = data.items || [];
  } catch {/* silent */}
  const now = new Date().toISOString();
  const urls = [
    ...STATIC_PAGES.map(({p,cf,pr}) =>
      `  <url><loc>${ORIGIN}${p}</loc><lastmod>${now}</lastmod><changefreq>${cf}</changefreq><priority>${pr}</priority></url>`),
    ...items.map((a) => {
      const lm = esc(a.published_at || a.collected_at || now);
      return `  <url><loc>${ORIGIN}/articles/${a.id}</loc><lastmod>${lm}</lastmod><changefreq>monthly</changefreq><priority>0.6</priority></url>`;
    })
  ];
  const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${urls.join('\n')}
</urlset>`;
  return new Response(xml, {
    headers: {
      'Content-Type': 'application/xml; charset=utf-8',
      'Cache-Control': 'public, max-age=3600'
    }
  });
}

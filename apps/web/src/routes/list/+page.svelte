<script>
  let { data } = $props();
  let q = $state(data.q);

  function timeAgo(iso) {
    if (!iso) return '';
    const s = (Date.now() - new Date(iso).getTime()) / 1000;
    if (s < 60) return `${Math.floor(s)}초 전`;
    if (s < 3600) return `${Math.floor(s / 60)}분 전`;
    if (s < 86400) return `${Math.floor(s / 3600)}시간 전`;
    return `${Math.floor(s / 86400)}일 전`;
  }
</script>

<svelte:head><title>목록 · Ps Humor</title></svelte:head>

<div class="search-bar">
  <form method="get" action="/list">
    <input type="search" name="q" placeholder="키워드 검색 (예: 고양이, 짤)" bind:value={q} />
    <button type="submit">검색</button>
  </form>
  <p class="hint">의미 검색은 <a href="/search">여기</a>에서.</p>
</div>

<section>
  <h2>{data.q ? `"${data.q}" 검색 결과` : '최근 게시물'}</h2>
  <p class="count">{data.articles.length}건</p>

  <div class="grid">
    {#each data.articles as a (a.id)}
      <article class="card">
        {#if a.image_url}
          <img src={a.image_url} alt="" loading="lazy" />
        {/if}
        <div class="body">
          <div class="meta">
            <span class="source">{a.source_name}</span>
            <span class="time">{timeAgo(a.published_at || a.collected_at)}</span>
          </div>
          <h3>
            <a href={a.url} target="_blank" rel="noopener noreferrer">{a.translated_title || a.title}</a>
          </h3>
          {#if a.translated_title && a.language && a.language !== 'ko'}
            <p class="orig-title">{a.title} <span class="lang">[{a.language.toUpperCase()}]</span></p>
          {/if}
          {#if a.translated_summary || a.summary}
            <p class="summary">{(a.translated_summary || a.summary).slice(0, 140)}{(a.translated_summary || a.summary).length > 140 ? '…' : ''}</p>
          {/if}
          <a class="more" href="/articles/{a.id}">자세히</a>
        </div>
      </article>
    {/each}
  </div>
</section>

<style>
  .search-bar form { display: flex; gap: 8px; margin-bottom: 8px; }
  .search-bar input { flex: 1; padding: 10px 14px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 14px; }
  .search-bar button { padding: 10px 20px; background: #2563eb; color: #fff; border: 0; border-radius: 6px; font-weight: 600; cursor: pointer; }
  .hint { color: #6b7280; font-size: 13px; margin: 4px 0 24px; }
  h2 { margin: 16px 0 4px; font-size: 18px; }
  .count { color: #6b7280; font-size: 13px; margin: 0 0 16px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px; }
  .card { background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.04); display: flex; flex-direction: column; }
  .card img { width: 100%; aspect-ratio: 16 / 9; object-fit: cover; background: #f3f4f6; }
  .body { padding: 12px 16px 14px; flex: 1; display: flex; flex-direction: column; }
  .meta { display: flex; justify-content: space-between; font-size: 12px; color: #6b7280; margin-bottom: 6px; }
  .source { font-weight: 600; color: #2563eb; }
  h3 { margin: 0 0 8px; font-size: 15px; line-height: 1.4; }
  h3 a:hover { text-decoration: underline; }
  .orig-title { font-size: 12px; color: #9ca3af; margin: -4px 0 8px; line-height: 1.4; }
  .lang { font-weight: 600; font-size: 11px; color: #6b7280; }
  .summary { color: #4b5563; font-size: 13px; line-height: 1.5; margin: 0 0 8px; flex: 1; }
  .more { font-size: 12px; color: #6b7280; }
</style>

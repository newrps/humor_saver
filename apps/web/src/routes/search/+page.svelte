<script>
  let { data } = $props();
  let q = $state(data.q);
</script>

<svelte:head><title>의미 검색 · Ps Humor</title></svelte:head>

<div class="search-bar">
  <form method="get" action="/search">
    <input type="search" name="q" placeholder="자연어로 검색 (예: 고양이, 사기 결혼, 사고)" bind:value={q} autofocus />
    <button type="submit">의미 검색</button>
  </form>
  <p class="hint">apple 로 검색해도 사과 게시물이 나옵니다 (다국어 벡터 검색).</p>
</div>

{#if data.error}
  <div class="error">에러: {data.error}</div>
{/if}

{#if data.q}
  <h2>"{data.q}" 의미 검색 결과 — {data.results.length}건</h2>

  <ul class="results">
    {#each data.results as r (r.article_id)}
      <li class="hit">
        <div class="score">{(r.score * 100).toFixed(1)}<span>%</span></div>
        {#if r.image_url}<img src={r.image_url} alt="" loading="lazy" />{/if}
        <div class="body">
          <div class="meta">
            <span class="source">{r.source_name || ''}</span>
            <span class="time">{r.published_at ? new Date(r.published_at).toLocaleString('ko-KR') : ''}</span>
          </div>
          <h3>
            <a href={r.url} target="_blank" rel="noopener noreferrer">{r.translated_title || r.title}</a>
          </h3>
          {#if r.translated_title && r.language && r.language !== 'ko'}
            <p class="orig-title">{r.title} <span class="lang">[{r.language.toUpperCase()}]</span></p>
          {/if}
          {#if r.translated_summary || r.summary}
            <p>{(r.translated_summary || r.summary).slice(0, 200)}{(r.translated_summary || r.summary).length > 200 ? '…' : ''}</p>
          {/if}
        </div>
      </li>
    {/each}
  </ul>
{:else}
  <div class="empty">위 검색창에 자연어 질의를 입력하세요.</div>
{/if}

<style>
  .search-bar form { display: flex; gap: 8px; margin-bottom: 8px; }
  .search-bar input { flex: 1; padding: 12px 14px; border: 1px solid #d1d5db; border-radius: 6px; font-size: 14px; }
  .search-bar button { padding: 12px 22px; background: #2563eb; color: #fff; border: 0; border-radius: 6px; font-weight: 600; cursor: pointer; }
  .hint { color: #6b7280; font-size: 13px; margin: 4px 0 24px; }
  .error { color: #b91c1c; padding: 12px; background: #fee2e2; border-radius: 6px; margin: 16px 0; }
  h2 { font-size: 18px; margin: 16px 0; }
  .results { list-style: none; padding: 0; margin: 0; }
  .hit {
    background: #fff; border-radius: 8px; margin-bottom: 12px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.04);
    display: grid; grid-template-columns: 64px 120px 1fr; gap: 16px;
    padding: 14px; align-items: start;
  }
  .hit:not(:has(img)) { grid-template-columns: 64px 1fr; }
  .score { font-weight: 700; color: #2563eb; font-size: 18px; padding-top: 4px; }
  .score span { font-size: 11px; color: #6b7280; margin-left: 2px; }
  .hit img { width: 120px; aspect-ratio: 16/10; object-fit: cover; border-radius: 4px; }
  .body { min-width: 0; }
  .meta { display: flex; gap: 12px; font-size: 12px; color: #6b7280; margin-bottom: 4px; }
  .source { font-weight: 600; color: #2563eb; }
  h3 { margin: 0 0 6px; font-size: 15px; line-height: 1.4; }
  h3 a:hover { text-decoration: underline; }
  .orig-title { font-size: 12px; color: #9ca3af; margin: 0 0 6px; line-height: 1.4; }
  .lang { font-weight: 600; font-size: 11px; color: #6b7280; }
  .body p { color: #4b5563; font-size: 13px; line-height: 1.5; margin: 0; }
  .empty { color: #6b7280; padding: 40px; text-align: center; }
</style>

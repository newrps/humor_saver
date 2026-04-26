<script>
  let { data } = $props();
  let a = $derived(data.article);
</script>

<svelte:head><title>{a.title} · news-tracker</title></svelte:head>

<article>
  <div class="meta">
    <a class="source" href="/?source_id={a.source_id}">{a.source_name}</a>
    {#if a.author}<span>· {a.author}</span>{/if}
    {#if a.published_at}<span>· {new Date(a.published_at).toLocaleString('ko-KR')}</span>{/if}
  </div>
  <h1>{a.title}</h1>
  {#if a.image_url}<img src={a.image_url} alt="" />{/if}
  {#if a.summary}<p class="summary">{a.summary}</p>{/if}
  {#if a.content}
    <div class="content">{@html a.content.split('\n').map(l => `<p>${l}</p>`).join('')}</div>
  {/if}
  <div class="actions">
    <a class="btn" href={a.url} target="_blank" rel="noopener noreferrer">원문 보기 →</a>
    <a class="btn ghost" href="/">목록으로</a>
  </div>
</article>

<style>
  article { background: #fff; padding: 32px 40px; border-radius: 8px; max-width: 760px; margin: 0 auto; }
  .meta { font-size: 13px; color: #6b7280; margin-bottom: 8px; }
  .meta .source { font-weight: 600; color: #2563eb; }
  h1 { font-size: 24px; line-height: 1.3; margin: 0 0 20px; }
  img { width: 100%; border-radius: 6px; margin-bottom: 20px; }
  .summary { font-size: 16px; color: #374151; padding: 14px 16px; background: #f9fafb; border-radius: 6px; line-height: 1.6; margin-bottom: 20px; }
  .content { font-size: 15px; line-height: 1.7; color: #1f2937; }
  .content :global(p) { margin: 0 0 14px; }
  .actions { margin-top: 28px; display: flex; gap: 8px; }
  .btn { padding: 10px 18px; background: #2563eb; color: #fff; border-radius: 6px; font-size: 14px; }
  .btn.ghost { background: #f3f4f6; color: #4b5563; }
</style>

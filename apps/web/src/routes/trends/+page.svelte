<script>
  let { data } = $props();
  let max = $derived(data.keywords[0]?.article_count || 1);
</script>

<svelte:head><title>키워드 트렌드 · news-tracker</title></svelte:head>

<div class="header">
  <h2>최근 {data.days}일 키워드 TOP {data.keywords.length}</h2>
  <nav class="days">
    {#each [1, 3, 7, 14, 30] as d}
      <a href="/trends?days={d}" class:active={data.days === d}>{d}일</a>
    {/each}
  </nav>
</div>

<ul class="bars">
  {#each data.keywords as k, i (k.word)}
    <li>
      <span class="rank">{i + 1}</span>
      <a class="word" href="/?q={encodeURIComponent(k.word)}">{k.word}</a>
      <div class="bar-wrap">
        <div class="bar" style="width: {(k.article_count / max) * 100}%"></div>
      </div>
      <span class="count">{k.article_count}</span>
    </li>
  {/each}
</ul>

<style>
  .header { display: flex; justify-content: space-between; align-items: center; margin: 8px 0 24px; }
  h2 { font-size: 18px; margin: 0; }
  .days { display: flex; gap: 4px; }
  .days a { padding: 6px 12px; font-size: 13px; color: #6b7280; border-radius: 4px; }
  .days a.active { background: #2563eb; color: #fff; }
  .bars { list-style: none; padding: 0; margin: 0; }
  .bars li {
    display: grid; grid-template-columns: 32px 100px 1fr 50px;
    gap: 12px; align-items: center;
    padding: 8px 0; border-bottom: 1px solid #e5e7eb;
  }
  .rank { color: #9ca3af; font-size: 12px; text-align: right; }
  .word { font-weight: 600; }
  .word:hover { color: #2563eb; }
  .bar-wrap { background: #f3f4f6; height: 8px; border-radius: 4px; overflow: hidden; }
  .bar { background: #2563eb; height: 100%; }
  .count { text-align: right; font-size: 13px; color: #6b7280; font-variant-numeric: tabular-nums; }
</style>

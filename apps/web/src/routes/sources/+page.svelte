<script>
  let { data } = $props();
  // 카테고리별 그룹
  let grouped = $derived(
    data.sources.reduce((m, s) => {
      (m[s.category] ||= []).push(s);
      return m;
    }, {})
  );
  const categoryNames = {
    general_daily: '종합 일간지', economy: '경제지', broadcast: '방송사',
    wire: '통신사', internet: '인터넷', tech: 'IT/테크', sports: '스포츠',
    entertainment: '연예', local: '지역', specialized: '전문지', english: '영문'
  };
</script>

<svelte:head><title>매체 · news-tracker</title></svelte:head>

<h2>등록 매체 ({data.sources.length}개)</h2>

{#each Object.entries(grouped) as [cat, list]}
  <section class="cat">
    <h3>{categoryNames[cat] || cat} <span>({list.length})</span></h3>
    <ul>
      {#each list as s (s.id)}
        <li class:disabled={!s.enabled} class:erroring={s.consecutive_errors > 3}>
          <a href="/?source_id={s.id}">{s.name}</a>
          <span class="status">
            {#if !s.enabled}비활성
            {:else if s.consecutive_errors > 3}오류 {s.consecutive_errors}회
            {:else if s.last_success_at}최근 수집 OK
            {:else}대기
            {/if}
          </span>
        </li>
      {/each}
    </ul>
  </section>
{/each}

<style>
  h2 { font-size: 18px; margin: 8px 0 24px; }
  .cat { margin-bottom: 24px; }
  .cat h3 { font-size: 14px; color: #4b5563; margin: 0 0 8px; padding-bottom: 4px; border-bottom: 1px solid #e5e7eb; }
  .cat h3 span { color: #9ca3af; font-weight: 400; }
  ul { list-style: none; padding: 0; margin: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 8px; }
  li { background: #fff; padding: 8px 12px; border-radius: 4px; display: flex; justify-content: space-between; align-items: center; font-size: 13px; }
  li.disabled { opacity: 0.4; }
  li.erroring { background: #fef2f2; }
  li a { font-weight: 500; }
  li a:hover { color: #2563eb; }
  .status { font-size: 11px; color: #9ca3af; }
  li.erroring .status { color: #b91c1c; }
</style>

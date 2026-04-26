<script>
  import { onMount } from 'svelte';
  let { data } = $props();
  let articles = $state([...data.articles]);
  let nextOffset = $state(data.offset + data.limit);
  let loading = $state(false);
  let done = $state(false);
  let feedEl;

  function timeAgo(iso) {
    if (!iso) return '';
    const s = (Date.now() - new Date(iso).getTime()) / 1000;
    if (s < 60) return `${Math.floor(s)}초 전`;
    if (s < 3600) return `${Math.floor(s / 60)}분 전`;
    if (s < 86400) return `${Math.floor(s / 3600)}시간 전`;
    return `${Math.floor(s / 86400)}일 전`;
  }

  // intersection observer로 마지막 카드 도달 감지 → 다음 페이지 로드
  onMount(() => {
    if (!feedEl) return;
    const obs = new IntersectionObserver(async (entries) => {
      for (const e of entries) {
        if (e.isIntersecting && !loading && !done) {
          loading = true;
          try {
            const r = await fetch(`/feed/more?offset=${nextOffset}&limit=20`);
            if (r.ok) {
              const j = await r.json();
              if (j.items.length === 0) {
                done = true;
              } else {
                articles = [...articles, ...j.items];
                nextOffset += j.items.length;
              }
            }
          } catch (err) {
            console.error('load more failed', err);
          }
          loading = false;
        }
      }
    }, { root: feedEl, rootMargin: '300px' });

    const updateObserve = () => {
      const cards = feedEl.querySelectorAll('.card');
      if (cards.length >= 3) {
        obs.disconnect();
        obs.observe(cards[cards.length - 3]);
      }
    };
    updateObserve();
    const mut = new MutationObserver(updateObserve);
    mut.observe(feedEl, { childList: true });

    return () => { obs.disconnect(); mut.disconnect(); };
  });
</script>

<svelte:head><title>피드 · Ps Humor</title></svelte:head>

<div class="feed" bind:this={feedEl}>
  {#each articles as a (a.id)}
    <article class="card">
      {#if a.image_url}
        <div class="bg" style="background-image: url('{a.image_url}')"></div>
      {/if}
      <div class="overlay">
        <div class="meta-top">
          <span class="source">{a.source_name || ''}</span>
          <span class="time">{timeAgo(a.published_at || a.collected_at)}</span>
        </div>
        <div class="bottom">
          <h2>{a.title}</h2>
          {#if a.summary}
            <p class="summary">{a.summary.length > 200 ? a.summary.slice(0, 200) + '…' : a.summary}</p>
          {/if}
          <a class="link" href={a.url} target="_blank" rel="noopener noreferrer">원문 →</a>
        </div>
      </div>
    </article>
  {/each}

  {#if loading}<div class="loading">로딩…</div>{/if}
  {#if done}<div class="loading">— 끝 —</div>{/if}

  {#if articles.length === 0}
    <div class="empty">
      <p>아직 수집된 콘텐츠가 없습니다.</p>
      <p>잠시 후 다시 시도해주세요.</p>
    </div>
  {/if}
</div>

<style>
  .feed {
    height: 100vh; height: 100dvh;
    overflow-y: scroll;
    scroll-snap-type: y mandatory;
    scrollbar-width: none;
    -ms-overflow-style: none;
    overscroll-behavior: contain;
  }
  .feed::-webkit-scrollbar { display: none; }

  .card {
    height: 100vh; height: 100dvh;
    scroll-snap-align: start;
    scroll-snap-stop: always;
    position: relative;
    background: #18181b;
    overflow: hidden;
  }
  .bg {
    position: absolute; inset: 0;
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    filter: brightness(0.5);
  }
  .overlay {
    position: absolute; inset: 0;
    display: flex; flex-direction: column;
    justify-content: space-between;
    padding: 80px 24px 40px;
    color: #fff;
  }
  .meta-top {
    display: flex; gap: 12px;
    font-size: 13px;
  }
  .source {
    background: rgba(245, 158, 11, 0.95);
    color: #000;
    padding: 4px 10px;
    border-radius: 12px;
    font-weight: 700;
  }
  .time {
    color: #d4d4d8;
    align-self: center;
    text-shadow: 0 1px 4px rgba(0,0,0,0.7);
  }
  .bottom {
    background: linear-gradient(to top, rgba(0,0,0,0.85) 0%, rgba(0,0,0,0.3) 70%, transparent 100%);
    margin: 0 -24px -40px;
    padding: 60px 24px 40px;
  }
  h2 {
    margin: 0 0 12px;
    font-size: 22px;
    line-height: 1.35;
    text-shadow: 0 2px 8px rgba(0,0,0,0.8);
    word-break: keep-all;
  }
  .summary {
    margin: 0 0 16px;
    font-size: 14px;
    color: #e4e4e7;
    line-height: 1.6;
    text-shadow: 0 1px 4px rgba(0,0,0,0.8);
  }
  .link {
    display: inline-block;
    padding: 10px 18px;
    background: rgba(255,255,255,0.15);
    border: 1px solid rgba(255,255,255,0.3);
    border-radius: 24px;
    font-size: 14px;
    backdrop-filter: blur(8px);
  }
  .link:hover { background: rgba(255,255,255,0.25); }

  .loading, .empty {
    height: 100vh; height: 100dvh;
    display: flex; align-items: center; justify-content: center;
    color: #a1a1aa;
    scroll-snap-align: start;
  }
  .empty { flex-direction: column; gap: 8px; }
</style>

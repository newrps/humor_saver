<script>
  import { page } from '$app/stores';
  let { data, children } = $props();
  let path = $derived($page.url.pathname);
  // 메인 피드(/)는 풀스크린, 나머지는 일반 레이아웃
  let isFeed = $derived(path === '/');
</script>

<header class:transparent={isFeed}>
  <div class="inner">
    <a href="/" class="logo">😆 Ps Humor</a>
    <nav>
      <a href="/" class:active={path === '/'}>피드</a>
      <a href="/list" class:active={path === '/list'}>목록</a>
      <a href="/search" class:active={path.startsWith('/search')}>검색</a>
      <a href="/trends" class:active={path === '/trends'}>트렌드</a>
      {#if data?.isHomeIp}
        <a href="/sources" class:active={path === '/sources'}>매체</a>
      {/if}
    </nav>
  </div>
</header>

<main class:full={isFeed}>
  {@render children()}
</main>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    overscroll-behavior: none;
  }
  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Malgun Gothic', sans-serif;
    background: #0b0b0f;
    color: #f5f5f7;
  }
  :global(a) { color: inherit; text-decoration: none; }
  header {
    background: rgba(11, 11, 15, 0.85);
    backdrop-filter: blur(8px);
    border-bottom: 1px solid #1f1f25;
    position: fixed; top: 0; left: 0; right: 0; z-index: 10;
  }
  header.transparent {
    background: linear-gradient(to bottom, rgba(0,0,0,0.6) 0%, transparent 100%);
    border-bottom: none;
  }
  .inner {
    max-width: 1100px; margin: 0 auto;
    padding: 14px 24px;
    display: flex; align-items: center; gap: 32px;
  }
  .logo { font-weight: 700; font-size: 18px; color: #f5f5f7; }
  nav { display: flex; gap: 18px; }
  nav a {
    padding: 6px 4px;
    color: #a1a1aa;
    border-bottom: 2px solid transparent;
    font-size: 14px;
  }
  nav a.active { color: #fff; border-bottom-color: #f59e0b; }
  main {
    max-width: 1100px; margin: 0 auto;
    padding: 80px 24px 24px;
  }
  main.full {
    max-width: none; margin: 0; padding: 0;
    height: 100vh; height: 100dvh;
    overflow: hidden;
  }
</style>

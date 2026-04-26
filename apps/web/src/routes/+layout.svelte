<script>
  import { page } from '$app/stores';
  let { data, children } = $props();
  let path = $derived($page.url.pathname);
</script>

<header>
  <div class="inner">
    <a href="/" class="logo">📰 Ps News</a>
    <nav>
      <a href="/" class:active={path === '/'}>최근 기사</a>
      <a href="/search" class:active={path.startsWith('/search')}>의미 검색</a>
      <a href="/trends" class:active={path === '/trends'}>키워드 트렌드</a>
      {#if data?.isHomeIp}
        <a href="/sources" class:active={path === '/sources'}>매체</a>
      {/if}
    </nav>
  </div>
</header>

<main>
  {@render children()}
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Malgun Gothic', sans-serif;
    background: #f5f6f8;
    color: #1a1a1a;
  }
  :global(a) { color: inherit; text-decoration: none; }
  header {
    background: #fff;
    border-bottom: 1px solid #e5e7eb;
    position: sticky; top: 0; z-index: 10;
  }
  .inner {
    max-width: 1100px; margin: 0 auto;
    padding: 14px 24px;
    display: flex; align-items: center; gap: 32px;
  }
  .logo { font-weight: 700; font-size: 18px; }
  nav { display: flex; gap: 18px; }
  nav a {
    padding: 6px 4px;
    color: #4b5563;
    border-bottom: 2px solid transparent;
    font-size: 14px;
  }
  nav a.active { color: #2563eb; border-bottom-color: #2563eb; }
  main {
    max-width: 1100px; margin: 0 auto;
    padding: 24px;
  }
</style>

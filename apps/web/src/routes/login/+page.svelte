<script>
  let { form } = $props();
</script>

<svelte:head><title>로그인 · Ps News</title></svelte:head>

<div class="page">
  <div class="card">
    <h1>📰 Ps News</h1>
    <p class="hint">접속하려면 비밀번호를 입력하세요.</p>

    <form method="POST">
      <input
        type="password"
        name="password"
        placeholder="비밀번호"
        autocomplete="current-password"
        autofocus
        required
      />
      <button type="submit">로그인</button>
    </form>

    {#if form?.error}
      <p class="error">{form.error}</p>
    {/if}
  </div>
</div>

<style>
  .page {
    min-height: 70vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .card {
    background: #fff;
    padding: 32px 40px;
    border-radius: 10px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.06);
    width: 100%;
    max-width: 360px;
  }
  h1 {
    margin: 0 0 8px;
    font-size: 22px;
    text-align: center;
  }
  .hint {
    color: #6b7280;
    font-size: 13px;
    text-align: center;
    margin: 0 0 20px;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  input {
    padding: 12px 14px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 14px;
  }
  button {
    padding: 12px;
    background: #2563eb;
    color: #fff;
    border: 0;
    border-radius: 6px;
    font-weight: 600;
    cursor: pointer;
    font-size: 14px;
  }
  button:hover { background: #1d4ed8; }
  .error {
    color: #b91c1c;
    background: #fee2e2;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 13px;
    margin: 12px 0 0;
    text-align: center;
  }
</style>

import { fail, redirect } from '@sveltejs/kit';
import { COOKIE_NAME, COOKIE_VALUE, PASSWORD } from '../../hooks.server.js';

export const actions = {
  default: async ({ request, cookies }) => {
    const data = await request.formData();
    const password = (data.get('password') || '').toString();

    if (!PASSWORD) {
      // .env 에 SITE_PASSWORD 가 없으면 인증 비활성 → 그냥 통과
      throw redirect(303, '/');
    }

    if (password !== PASSWORD) {
      return fail(401, { error: '비밀번호가 일치하지 않습니다' });
    }

    cookies.set(COOKIE_NAME, COOKIE_VALUE, {
      path: '/',
      httpOnly: true,
      sameSite: 'lax',
      secure: false,                // reverse proxy(HTTPS) 뒤에선 true 권장
      maxAge: 60 * 60 * 24 * 30      // 30일
    });

    throw redirect(303, '/');
  }
};

// 접근 제어
//
// 1. 사이트 전체 보호: SITE_PASSWORD 쿠키 또는 LAN/집 IP 만 통과
// 2. /sources: 더 엄격 — LAN/집 IP 만 (비밀번호 통과로는 X)

import crypto from 'node:crypto';

const HOME_IPS = (process.env.HOME_IPS || '')
  .split(',')
  .map((s) => s.trim())
  .filter(Boolean);

const SITE_PASSWORD = (process.env.SITE_PASSWORD || '').trim();
// 비번이 .env 에 없으면 인증 비활성 (개발 편의용)
const AUTH_DISABLED = SITE_PASSWORD === '';

// 쿠키 값 = sha256(SITE_PASSWORD + salt). 비번 바뀌면 자동 무효화.
const AUTH_COOKIE_NAME = 'ps_auth';
const AUTH_COOKIE_VALUE = SITE_PASSWORD
  ? crypto.createHash('sha256').update(SITE_PASSWORD + 'ps-news-salt').digest('hex')
  : '';

export const COOKIE_NAME = AUTH_COOKIE_NAME;
export const COOKIE_VALUE = AUTH_COOKIE_VALUE;
export const PASSWORD = SITE_PASSWORD;

function getRequestHost(event) {
  const raw =
    event.request.headers.get('x-forwarded-host') ||
    event.request.headers.get('host') ||
    event.url.hostname;
  return (raw || '').split(':')[0].toLowerCase();
}

function isLanHost(host) {
  if (!host) return false;
  return (
    host === 'localhost' ||
    host === '127.0.0.1' ||
    host.startsWith('192.168.') ||
    host.startsWith('10.') ||
    /^172\.(1[6-9]|2\d|3[01])\./.test(host)
  );
}

function getClientIp(event) {
  const headerIp =
    event.request.headers.get('cf-connecting-ip') ||
    event.request.headers.get('x-real-ip') ||
    event.request.headers.get('x-forwarded-for')?.split(',')[0].trim();
  if (headerIp) return { ip: headerIp, behindProxy: true };
  try {
    return { ip: event.getClientAddress(), behindProxy: false };
  } catch {
    return { ip: '', behindProxy: false };
  }
}

function isHomeIp({ ip, behindProxy }, host) {
  if (isLanHost(host)) return true;
  if (!behindProxy) return false;
  if (!ip || HOME_IPS.length === 0) return false;
  return HOME_IPS.includes(ip);
}

export async function handle({ event, resolve }) {
  const info = getClientIp(event);
  const host = getRequestHost(event);
  const homeIp = isHomeIp(info, host);

  const cookieAuth =
    !AUTH_DISABLED && event.cookies.get(AUTH_COOKIE_NAME) === AUTH_COOKIE_VALUE;
  const authenticated = AUTH_DISABLED || cookieAuth || homeIp;

  event.locals.clientIp = info.ip;
  event.locals.requestHost = host;
  event.locals.isHomeIp = homeIp;
  event.locals.authenticated = authenticated;

  const path = event.url.pathname;

  // 정적 자산은 무조건 통과
  if (path.startsWith('/_app/') || path === '/favicon.ico') {
    return resolve(event);
  }

  // /login 페이지: 인증된 사용자는 / 로 리디렉트, 아니면 통과
  if (path === '/login') {
    if (authenticated) {
      return new Response(null, { status: 303, headers: { location: '/' } });
    }
    return resolve(event);
  }

  // /logout: 쿠키 제거 후 /login 으로
  if (path === '/logout') {
    event.cookies.delete(AUTH_COOKIE_NAME, { path: '/' });
    return new Response(null, { status: 303, headers: { location: '/login' } });
  }

  // 그 외 모든 경로: 인증 안 됐으면 /login 으로
  if (!authenticated) {
    return new Response(null, { status: 303, headers: { location: '/login' } });
  }

  // /sources 는 IP 화이트리스트만 (비밀번호로는 통과 X)
  if (path.startsWith('/sources') && !homeIp) {
    return new Response('Not Found', { status: 404 });
  }

  return resolve(event);
}

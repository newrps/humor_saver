// 모든 페이지 layout 으로 isHomeIp 전달 (메뉴 표시 결정용)
export async function load({ locals }) {
  return {
    isHomeIp: locals.isHomeIp ?? false
  };
}

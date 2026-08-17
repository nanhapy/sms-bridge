/**
 * 验证码提取工具
 * strict 模式：跳过「纯数字」兜底规则（用于剪贴板场景，避免把普通 6 位数字误判为验证码），
 * 但允许剪贴板内容恰好是纯 4~8 位数字（对应信息 App 通知栏「复制」按钮的行为）。
 */

const PATTERNS: RegExp[] = [
  /(?:验证码|校验码|动态验证码|动态码|安全码|确认码|短信码)\s*(?:是|为|:|：|，|,)?\s*[^\d]{0,10}?(\d{4,8})/,
  /(?:验证码|校验码|动态验证码|动态码|安全码|确认码|短信码)[^\d]{0,20}?(\d{4,8})/,
  /(?:code|verification\s*code|otp)[^\d]{0,20}?(\d{4,8})/i,
  /(\d{4,8})[^\d]{0,10}?(?:验证码|校验码|动态验证码|动态码|安全码|短信码)/,
  /(?:【|\[)(\d{4,8})(?:】|\])/,
  /\b(\d{6})\b/,
  /\b(\d{4})\b/,
];

export function extractCode(text: string, strict: boolean = false): string | null {
  if (!text || text.length === 0) {
    return null;
  }
  // 常规规则（含关键词 / 方括号形式）
  for (let i = 0; i < 5; i++) {
    const m = text.match(PATTERNS[i]);
    if (m && m[1]) {
      return m[1];
    }
  }
  if (strict) {
    // 严格模式：只接受「整段内容就是纯数字」的情况
    const t = text.trim();
    if (/^\d{4,8}$/.test(t)) {
      return t;
    }
    return null;
  }
  // 非严格模式：纯 6 位 / 纯 4 位兜底
  for (let i = 5; i < PATTERNS.length; i++) {
    const m = text.match(PATTERNS[i]);
    if (m && m[1]) {
      return m[1];
    }
  }
  return null;
}

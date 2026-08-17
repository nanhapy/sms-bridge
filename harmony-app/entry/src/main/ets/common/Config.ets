/**
 * 配置存取（手机端）：电脑 IP、端口、剪贴板监听开关、最近推送记录
 * 使用 preferences 持久化，UI 与分享扩展共用同一份配置。
 */
import { preferences } from '@kit.ArkData';
import { common } from '@kit.AbilityKit';

const STORE_NAME = 'sms_bridge';
const KEY_IP = 'pc_ip';
const KEY_PORT = 'pc_port';
const KEY_WATCH = 'watch_clipboard';
const KEY_LAST_CODE = 'last_code';
const KEY_LAST_TIME = 'last_time';

export class Config {
  private static getPrefs(ctx: common.Context): preferences.Preferences {
    return preferences.getPreferencesSync(ctx, { name: STORE_NAME });
  }

  static getIp(ctx: common.Context): string {
    const p = Config.getPrefs(ctx);
    return p.getSync(KEY_IP, '') as string;
  }

  static setIp(ctx: common.Context, ip: string): void {
    const p = Config.getPrefs(ctx);
    p.putSync(KEY_IP, ip);
    p.flush();
  }

  static getPort(ctx: common.Context): string {
    const p = Config.getPrefs(ctx);
    return p.getSync(KEY_PORT, '8899') as string;
  }

  static setPort(ctx: common.Context, port: string): void {
    const p = Config.getPrefs(ctx);
    p.putSync(KEY_PORT, port);
    p.flush();
  }

  static getWatch(ctx: common.Context): boolean {
    const p = Config.getPrefs(ctx);
    return p.getSync(KEY_WATCH, true) as boolean;
  }

  static setWatch(ctx: common.Context, on: boolean): void {
    const p = Config.getPrefs(ctx);
    p.putSync(KEY_WATCH, on);
    p.flush();
  }

  /** 返回 [上次验证码, 上次时间戳] */
  static getLast(ctx: common.Context): string[] {
    const p = Config.getPrefs(ctx);
    const code = p.getSync(KEY_LAST_CODE, '') as string;
    const time = p.getSync(KEY_LAST_TIME, 0) as number;
    return [code, String(time)];
  }

  static setLast(ctx: common.Context, code: string): void {
    const p = Config.getPrefs(ctx);
    p.putSync(KEY_LAST_CODE, code);
    p.putSync(KEY_LAST_TIME, Date.now());
    p.flush();
  }
}

/**
 * 向电脑接收端发起 HTTP 请求：健康检查 + 验证码推送
 */
import { http } from '@kit.NetworkKit';
import { BusinessError } from '@kit.BasicServicesKit';

export interface PushResult {
  ok: boolean;
  message: string;
  code: string;
}

interface HealthResp {
  ok?: boolean;
  host?: string;
  port?: number;
}

interface PushResp {
  ok?: boolean;
  code?: string;
  repeated?: boolean;
}

/** 推送给电脑的请求体（ArkTS 要求对象字面量对应显式声明的接口） */
interface PushBody {
  text: string;
  code: string;
  source: string;
  time: number;
}

export async function pingPc(ip: string, port: string): Promise<PushResult> {
  const req = http.createHttp();
  try {
    const resp = await req.request(`http://${ip}:${port}/health`, {
      method: http.RequestMethod.GET,
      connectTimeout: 4000,
      readTimeout: 4000,
    });
    if (resp.responseCode === 200) {
      const data = JSON.parse(resp.result as string) as HealthResp;
      const host: string = data.host ?? '电脑';
      return { ok: true, message: `连接成功：${host}`, code: '' };
    }
    return { ok: false, message: `电脑返回 HTTP ${resp.responseCode}`, code: '' };
  } catch (e) {
    const err = e as BusinessError;
    return { ok: false, message: `连接失败：${err.message ?? '未知错误'}`, code: '' };
  } finally {
    req.destroy();
  }
}

export async function pushToPc(ip: string, port: string, text: string, code: string,
  source: string): Promise<PushResult> {
  const req = http.createHttp();
  try {
    const body: PushBody = {
      text: text,
      code: code,
      source: source,
      time: Date.now(),
    };
    const resp = await req.request(`http://${ip}:${port}/push`, {
      method: http.RequestMethod.POST,
      header: { 'Content-Type': 'application/json' },
      extraData: JSON.stringify(body),
      connectTimeout: 4000,
      readTimeout: 4000,
    });
    if (resp.responseCode === 200) {
      const data = JSON.parse(resp.result as string) as PushResp;
      if (data.repeated === true) {
        return { ok: true, message: '重复推送（电脑已忽略）', code: code };
      }
      return { ok: true, message: '已推送到电脑并复制', code: code };
    }
    return { ok: false, message: `电脑返回 HTTP ${resp.responseCode}`, code: '' };
  } catch (e) {
    const err = e as BusinessError;
    return { ok: false, message: `连接失败：${err.message ?? '未知错误'}`, code: '' };
  } finally {
    req.destroy();
  }
}

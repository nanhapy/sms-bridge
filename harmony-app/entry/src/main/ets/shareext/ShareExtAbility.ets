/**
 * 分享扩展（API 12 规范）：接入系统分享面板的「分享详情页」
 *
 * 官方流程（华为 Share Kit 文档《分享详情页处理分享内容》）：
 *   1. 用户在分享面板选择本 App → 系统创建 ShareExtensionAbility
 *   2. onSessionCreate(want, session) 被调用
 *   3. 通过 systemShare.getSharedData(want) 解析分享数据（SharedRecord.utd / .content）
 *   4. session.loadContent() 渲染分享详情页（由目标 App 自己绘制）
 *   5. 处理完成后 session.terminateSelfWithResult({ resultCode: ShareAbilityResultCode.CLOSE }) 关闭
 */
import { ShareExtensionAbility, UIExtensionContentSession, Want } from '@kit.AbilityKit';
import { systemShare } from '@kit.ShareKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { BusinessError } from '@kit.BasicServicesKit';
import { extractCode } from '../common/CodeParser';
import { Config } from '../common/Config';
import { pushToPc } from '../common/HttpPush';

const DOMAIN = 0x0001;
const REPEAT_WINDOW_MS = 60000;
const CLOSE_DELAY_MS = 1500; // 详情页停留时间，让用户能看到结果

export default class ShareExtAbility extends ShareExtensionAbility {
  onSessionCreate(want: Want, session: UIExtensionContentSession): void {
    // 先渲染分享详情页（系统要求目标应用绘制该页面）
    session.loadContent('pages/SharePanel');
    this.handleShare(want, session);
  }

  private async handleShare(want: Want, session: UIExtensionContentSession): Promise<void> {
    const ctx = this.context;
    let pushed = false;
    try {
      // 1. 解析分享数据
      const data = await systemShare.getSharedData(want);
      let text = '';
      const records = data.getRecords();
      for (const record of records) {
        const utd = record.utd as string;
        const content = record.content as string;
        if (utd && utd.indexOf('text') >= 0 && content && content.length > 0) {
          text = content;
          break;
        }
      }
      if (!text || text.length === 0) {
        hilog.info(DOMAIN, 'SMSBridge', '分享内容里没有文本');
        return;
      }

      // 2. 校验配置
      const ip = Config.getIp(ctx);
      const port = Config.getPort(ctx);
      if (!ip || ip.length === 0) {
        hilog.warn(DOMAIN, 'SMSBridge', '未配置电脑 IP，忽略本次分享');
        return;
      }

      // 3. 提取验证码并去重
      const code = extractCode(text, false);
      const last = Config.getLast(ctx);
      const lastCode = last[0];
      const lastTime = Number(last[1]);
      if (code && code === lastCode && Date.now() - lastTime < REPEAT_WINDOW_MS) {
        hilog.info(DOMAIN, 'SMSBridge', '60 秒内相同验证码，跳过');
        pushed = true; // 视为已处理，快速关闭
        return;
      }

      // 4. 推送到电脑
      const result = await pushToPc(ip, port, text, code ?? '', 'share');
      hilog.info(DOMAIN, 'SMSBridge', '推送结果: %{public}s', result.message);
      if (result.ok) {
        Config.setLast(ctx, code ?? '');
        pushed = true;
      }
    } catch (e) {
      const err = e as BusinessError;
      hilog.error(DOMAIN, 'SMSBridge', '分享处理异常: %{public}s', err.message ?? '未知错误');
    } finally {
      // 停留片刻让用户看到详情页结果，然后关闭分享面板
      setTimeout(() => {
        session.terminateSelfWithResult({ resultCode: systemShare.ShareAbilityResultCode.CLOSE })
          .then(() => {
            hilog.info(DOMAIN, 'SMSBridge', '分享详情页已关闭');
          })
          .catch(() => {
            // 兜底：直接关闭
            session.terminateSelf().catch(() => {
              hilog.warn(DOMAIN, 'SMSBridge', '关闭分享详情页失败');
            });
          });
      }, pushed ? CLOSE_DELAY_MS : CLOSE_DELAY_MS * 2);
    }
  }
}

import { AbilityConstant, UIAbility, Want } from '@kit.AbilityKit';
import { systemShare } from '@kit.ShareKit';
import { hilog } from '@kit.PerformanceAnalysisKit';
import { BusinessError } from '@kit.BasicServicesKit';
import { window } from '@kit.ArkUI';
import { extractCode } from '../common/CodeParser';
import { Config } from '../common/Config';
import { pushToPc } from '../common/HttpPush';

const DOMAIN = 0x0000;
const REPEAT_WINDOW_MS = 60000;

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'SMSBridge', 'onCreate, launchReason: %{public}d', launchParam.launchReason);
    if (this.isSystemShare(launchParam)) {
      this.handleSharedData(want);
    }
  }

  /** 应用已在运行时被再次分享拉起 */
  onNewWant(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    hilog.info(DOMAIN, 'SMSBridge', 'onNewWant, launchReason: %{public}d', launchParam.launchReason);
    if (this.isSystemShare(launchParam)) {
      this.handleSharedData(want);
    }
  }

  private isSystemShare(launchParam: AbilityConstant.LaunchParam): boolean {
    return launchParam.launchReason === AbilityConstant.LaunchReason.SHARE ||
      launchParam.launchReasonMessage === 'ReasonMessage_SystemShare';
  }

  /** 处理系统分享面板分享来的数据（UIAbility 路径，AloeSend 同款机制） */
  private async handleSharedData(want: Want): Promise<void> {
    try {
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
        hilog.info(DOMAIN, 'SMSBridge', '分享数据中没有文本');
        return;
      }

      const ip = Config.getIp(this.context);
      const port = Config.getPort(this.context);
      if (!ip || ip.length === 0) {
        hilog.warn(DOMAIN, 'SMSBridge', '未配置电脑 IP，忽略本次分享');
        return;
      }

      const code = extractCode(text, false);
      const last = Config.getLast(this.context);
      const lastCode = last[0];
      const lastTime = Number(last[1]);
      if (code && code === lastCode && Date.now() - lastTime < REPEAT_WINDOW_MS) {
        hilog.info(DOMAIN, 'SMSBridge', '60 秒内相同验证码，跳过');
        return;
      }

      const result = await pushToPc(ip, port, text, code ?? '', 'share');
      hilog.info(DOMAIN, 'SMSBridge', '推送结果: %{public}s', result.message);
      if (result.ok) {
        Config.setLast(this.context, code ?? '');
      }
    } catch (e) {
      const err = e as BusinessError;
      hilog.error(DOMAIN, 'SMSBridge', '分享处理异常: %{public}s', err.message ?? '未知错误');
    }
  }

  onDestroy(): void {
    hilog.info(DOMAIN, 'SMSBridge', 'EntryAbility onDestroy');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        hilog.error(DOMAIN, 'SMSBridge', 'loadContent failed, code: %{public}d', err.code);
        return;
      }
      hilog.info(DOMAIN, 'SMSBridge', 'loadContent success');
    });
  }

  onWindowStageDestroy(): void {
    hilog.info(DOMAIN, 'SMSBridge', 'onWindowStageDestroy');
  }

  onForeground(): void {
    hilog.info(DOMAIN, 'SMSBridge', 'onForeground');
  }

  onBackground(): void {
    hilog.info(DOMAIN, 'SMSBridge', 'onBackground');
  }
}

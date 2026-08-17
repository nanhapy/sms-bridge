/**
 * 主界面：电脑地址配置、自动发现、测试连接、前台剪贴板监听、最近记录
 */
import { abilityAccessCtrl, common } from '@kit.AbilityKit';
import { promptAction } from '@kit.ArkUI';
import { pasteboard, BusinessError } from '@kit.BasicServicesKit';
import { extractCode } from '../common/CodeParser';
import { Config } from '../common/Config';
import { pushToPc, pingPc, PushResult } from '../common/HttpPush';
import { discoverPc } from '../common/UdpDiscovery';

const REPEAT_WINDOW_MS = 300000;
const MAX_LOGS = 20;

@Entry
@Component
struct Index {
  @State ip: string = '';
  @State port: string = '8899';
  @State watchClip: boolean = true;
  @State status: string = '待机';
  @State logs: string[] = [];
  private timer: number = -1;
  private sysPb: pasteboard.SystemPasteboard = pasteboard.getSystemPasteboard();
  private lastCode: string = '';
  private lastCodeTime: number = 0;
  private ctx: common.UIAbilityContext = getContext(this) as common.UIAbilityContext;
  private updateCb: () => void = (): void => {
    this.checkClipboard();
  };

  aboutToAppear(): void {
    this.ip = Config.getIp(this.ctx);
    this.port = Config.getPort(this.ctx);
    this.watchClip = Config.getWatch(this.ctx);
    this.timer = setInterval((): void => {
      this.checkClipboard();
    }, 1000);
    this.sysPb.on('update', this.updateCb);
    this.requestPasteboardPermission();
  }

  /** 剪贴板读取是 user_grant 权限，首次打开时向用户申请授权 */
  private requestPasteboardPermission(): void {
    const atManager = abilityAccessCtrl.createAtManager();
    atManager.requestPermissionsFromUser(this.ctx, ['ohos.permission.READ_PASTEBOARD'])
      .then((result) => {
        const granted = result.authResults.length > 0 && result.authResults[0] === 0;
        if (!granted) {
          this.addLog('未授权读取剪贴板，剪贴板监听不可用（分享方式不受影响）');
        }
      })
      .catch((e: BusinessError) => {
        this.addLog(`剪贴板权限申请失败: ${e.message ?? '未知错误'}`);
      });
  }

  aboutToDisappear(): void {
    if (this.timer >= 0) {
      clearInterval(this.timer);
      this.timer = -1;
    }
    this.sysPb.off('update', this.updateCb);
  }

  private now(): string {
    return new Date().toLocaleTimeString();
  }

  private addLog(msg: string): void {
    this.logs = [msg].concat(this.logs).slice(0, MAX_LOGS);
  }

  private checkClipboard(): void {
    if (!this.watchClip) {
      return;
    }
    this.sysPb.getData().then((data: pasteboard.PasteData) => {
      const text = data.getPrimaryText();
      if (!text || text.length === 0) {
        return;
      }
      const code = extractCode(text, true);
      if (!code) {
        return;
      }
      const nowMs = Date.now();
      if (code === this.lastCode && nowMs - this.lastCodeTime < REPEAT_WINDOW_MS) {
        return;
      }
      const ip = Config.getIp(this.ctx);
      const port = Config.getPort(this.ctx);
      if (!ip || ip.length === 0) {
        this.status = '请先配置电脑 IP 再复制验证码';
        return;
      }
      pushToPc(ip, port, text, code, 'clipboard').then((r: PushResult) => {
        this.lastCode = code;
        this.lastCodeTime = Date.now();
        this.status = r.ok ? `已推送验证码 ${code}` : `推送失败：${r.message}`;
        this.addLog(`[${this.now()}] ${this.status}`);
      });
    }).catch((e: BusinessError) => {
      this.addLog(`读取剪贴板失败: ${e.message ?? '未知错误'}`);
    });
  }

  private discover(): void {
    this.status = '正在自动发现电脑…';
    const portNum: number = Number(this.port) > 0 ? Number(this.port) : 8899;
    discoverPc(portNum, 3000,
      (foundIp: string, host: string, foundPort: string) => {
        this.ip = foundIp;
        this.port = foundPort;
        this.status = `发现电脑：${host}（${foundIp}）`;
        this.addLog(`[${this.now()}] ${this.status}`);
      },
      (): void => {
        if (this.status === '正在自动发现电脑…') {
          this.status = '未发现电脑，请确认接收端已启动且在同一 Wi-Fi，手动输入 IP';
          this.addLog(`[${this.now()}] ${this.status}`);
        }
      });
  }

  private save(): void {
    if (!this.ip || this.ip.length === 0) {
      promptAction.showToast({ message: '请先填写电脑 IP' });
      return;
    }
    Config.setIp(this.ctx, this.ip);
    Config.setPort(this.ctx, this.port.length > 0 ? this.port : '8899');
    this.status = '设置已保存';
    promptAction.showToast({ message: '设置已保存' });
  }

  private testConn(): void {
    if (!this.ip || this.ip.length === 0) {
      promptAction.showToast({ message: '请先填写电脑 IP' });
      return;
    }
    this.status = '正在测试连接…';
    const port: string = this.port.length > 0 ? this.port : '8899';
    pingPc(this.ip, port).then((r: PushResult) => {
      this.status = r.message;
      promptAction.showToast({ message: r.message });
      this.addLog(`[${this.now()}] ${r.message}`);
    });
  }

  /** 手动触发：读取剪贴板里的验证码并推送（带明确反馈，便于排查） */
  private manualClipboardPush(): void {
    this.sysPb.getData().then((data: pasteboard.PasteData) => {
      const text = data.getPrimaryText();
      if (!text || text.length === 0) {
        this.status = '剪贴板为空';
        promptAction.showToast({ message: '剪贴板为空，先去复制验证码' });
        return;
      }
      const code = extractCode(text, true);
      if (!code) {
        this.status = '剪贴板里没有识别到验证码';
        promptAction.showToast({ message: '剪贴板里没有验证码' });
        return;
      }
      const ip = Config.getIp(this.ctx);
      const port = Config.getPort(this.ctx);
      if (!ip || ip.length === 0) {
        this.status = '请先配置电脑 IP';
        promptAction.showToast({ message: '请先配置电脑 IP' });
        return;
      }
      pushToPc(ip, port, text, code, 'manual').then((r: PushResult) => {
        this.lastCode = code;
        this.lastCodeTime = Date.now();
        this.status = r.ok ? `已推送验证码 ${code}` : `推送失败：${r.message}`;
        promptAction.showToast({ message: this.status });
        this.addLog(`[${this.now()}] ${this.status}`);
      });
    }).catch((e: BusinessError) => {
      this.status = '读取剪贴板失败（可能未授权）';
      promptAction.showToast({ message: '读取剪贴板失败，请检查权限' });
      this.addLog(`[${this.now()}] 读取剪贴板失败: ${e.message ?? '未知错误'}`);
    });
  }

  build() {
    Column({ space: 10 }) {
      Text('验证码快传')
        .fontSize(24)
        .fontWeight(FontWeight.Bold)
        .margin({ top: 16 })
      Text('鸿蒙手机 → Windows 电脑（同一 Wi-Fi）')
        .fontSize(13)
        .fontColor('#909399')

      Column({ space: 10 }) {
        Row({ space: 8 }) {
          Text('电脑 IP')
            .fontSize(14)
            .width(60)
          TextInput({ placeholder: '例如 192.168.3.31', text: this.ip })
            .layoutWeight(1)
            .fontSize(14)
            .onChange((v: string) => {
              this.ip = v.trim();
            })
        }
        Row({ space: 8 }) {
          Text('端口')
            .fontSize(14)
            .width(60)
          TextInput({ placeholder: '8899', text: this.port })
            .layoutWeight(1)
            .fontSize(14)
            .type(InputType.Number)
            .onChange((v: string) => {
              this.port = v.trim();
            })
        }
        Row({ space: 8 }) {
          Button('自动发现')
            .layoutWeight(1)
            .fontSize(13)
            .onClick(() => {
              this.discover();
            })
          Button('保存')
            .layoutWeight(1)
            .fontSize(13)
            .onClick(() => {
              this.save();
            })
          Button('测试连接')
            .layoutWeight(1)
            .fontSize(13)
            .onClick(() => {
              this.testConn();
            })
        }
        .width('100%')
        Button('读取剪贴板并发送')
          .width('100%')
          .fontSize(14)
          .onClick(() => {
            this.manualClipboardPush();
          })
      }
      .width('92%')
      .padding(14)
      .backgroundColor('#F5F6FA')
      .borderRadius(12)

      Row({ space: 8 }) {
        Text('前台剪贴板监听')
          .fontSize(15)
          .layoutWeight(1)
        Toggle({ type: ToggleType.Switch, isOn: this.watchClip })
          .onChange((on: boolean) => {
            this.watchClip = on;
            Config.setWatch(this.ctx, on);
          })
      }
      .width('92%')

      Text(`状态：${this.status}`)
        .fontSize(14)
        .fontColor('#606266')
        .width('92%')

      Column({ space: 6 }) {
        Text('使用方法')
          .fontSize(15)
          .fontWeight(FontWeight.Medium)
        Text('方式一：收到短信 → 长按短信 → 分享 → 选「发送到电脑」')
          .fontSize(13)
          .fontColor('#606266')
        Text('方式二：通知栏点「复制」验证码 → 打开本 App 即可')
          .fontSize(13)
          .fontColor('#606266')
        Text('电脑端会弹窗并自动复制，桌面 App 里直接 Ctrl+V。')
          .fontSize(13)
          .fontColor('#606266')
      }
      .width('92%')
      .padding(12)
      .backgroundColor('#F5F6FA')
      .borderRadius(12)
      .alignItems(HorizontalAlign.Start)

      Column({ space: 4 }) {
        Text('最近记录')
          .fontSize(14)
          .fontColor('#909399')
          .width('100%')
        List({ space: 2 }) {
          ForEach(this.logs, (item: string, index: number) => {
            ListItem() {
              Text(item)
                .fontSize(12)
                .fontColor('#606266')
                .width('100%')
            }
          }, (item: string, index: number) => `${index}_${item}`)
        }
        .width('100%')
        .layoutWeight(1)
      }
      .width('92%')
      .layoutWeight(1)
      .alignItems(HorizontalAlign.Start)
      .margin({ bottom: 12 })
    }
    .width('100%')
    .height('100%')
  }
}

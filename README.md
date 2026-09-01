# 码到成功

码到成功将手机分享的验证码和文本发送到 Windows 接收端，接收端会显示通知并自动复制验证码。

> 名字化用成语“马到成功”——验证码（码）一到电脑上，事情就成了。

- Windows 接收端下载：[GitHub Releases](https://github.com/nanhapy/sms-bridge/releases)
- [隐私政策](https://nanhapy.github.io/sms-bridge/release/privacy-policy.html)

## 安装 Windows 接收端

1. 首次使用前，如旧版 Node 接收端仍在运行，请先由用户手动停止或卸载它。
2. 获取 `SMS Bridge Receiver_2.0.2_x64-setup.exe`，右键选择“以管理员身份运行”完成安装。安装过程需要提升权限，以配置 Windows 防火墙。
3. 安装后启动“码到成功”。接收端会在端口 8899 提供 TCP 接收和 UDP 发现服务。

安装器只添加 Private 网络配置文件下的 TCP/UDP 8899 入站规则；Public 网络不会开放这些规则。卸载时会删除这两条规则、当前用户的自启动项和本地应用数据。

## 使用接收端

首次启动默认启用开机自启动。窗口顶部的“自启动”开关可随时启用或关闭它。

关闭窗口不会退出接收端，而是隐藏到系统托盘。单击托盘图标可重新打开窗口；托盘菜单提供三个操作：

- 打开
- 清空历史记录
- 退出

收到消息后，接收端会自动提取验证码、复制验证码到剪贴板并显示系统通知。窗口保留最新 15 条记录，超过时可在历史区域滚动查看；点击整行即可再次复制验证码或消息内容。清空历史记录会先要求确认。

历史记录和首次启动配置以 JSON 文件保存在 `%APPDATA%\com.smsbridge.receiver`。这些是本机数据，不会上传到网络。

## 开发构建

开发环境需要 Windows、Node.js、Rust stable（MSVC 工具链）和 Visual Studio Build Tools 的 C++ 桌面开发组件。Windows 11 通常已安装 WebView2；安装包会在缺少时使用 WebView2 bootstrapper。

在仓库根目录执行：

```powershell
cd pc-receiver
npm install
npm run build
cargo check --manifest-path src-tauri\Cargo.toml
npm run tauri build
```

NSIS x64 安装包会生成到 `pc-receiver\src-tauri\target\release\bundle\nsis\`，例如 `SMS Bridge Receiver_2.0.0_x64-setup.exe`。

## 手机端人工验收

HarmonyOS 的实际验收由用户手动完成，构建成功不代表已验收。请在安装接收端后完成以下检查：

1. 确认托盘图标出现，且应用显示端口 8899 正在接收。
2. 用 HarmonyOS 应用发现电脑并发送一条真实消息，确认通知、自动复制、历史插入和整行复制。
3. 连续发送超过 15 条已接收消息，确认仅保留最新 15 条并可滚动查看。
4. 重启 Windows，确认接收端以仅托盘方式启动；关闭“自启动”后重启应用，确认该设置生效。
5. 验证关闭窗口后可从托盘重新打开、清空历史记录会要求确认，并可从托盘退出。
6. 让端口 8899 被其他程序占用后启动接收端，确认界面显示错误且 Tauri 进程仍在运行。

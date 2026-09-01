# 「码到成功」华为应用市场上架检查清单

> 时间线总览（各项并行推进）：
> - 第 1 天：启动 APP 备案、启动版权认证、AGC 创建应用、申请发布证书与 Profile
> - 第 1–2 天：配置发布签名、构建 .app、填写应用信息
> - 之后：等备案与版权材料下来 → 提审 → 常规审核约 24 小时
> - **总周期瓶颈是 APP 备案与版权材料，其余环节当天可完成**

## 一、立即启动（周期最长）

- [ ] **APP 备案（工信部）**：材料与话术见 `docs/release/备案材料.md`（含可直接粘贴的服务内容说明、材料清单、退回原因对照）。准备：实名认证账号、应用名称（码到成功）、包名、服务内容说明、隐私政策链接。注意：备案的应用名称、包名必须与 AGC 完全一致（包名现为 `com.smsbridge.app`）。
- [ ] **版权材料（二选一）**：软件著作权登记证书（常规 20–40 个工作日，可加急）或电子版权认证（数个工作日，AGC 认可）。

## 二、工程侧

- [x] 更换正式包名 `com.smsbridge.app`（AppScope/app.json5，2026-09-01 完成）
- [x] 统一应用名称为**码到成功**（手机端 label、主界面标题、PC 接收端界面与托盘、README、隐私政策、上架文案，2026-09-01 完成）
- [x] deviceTypes 收敛为仅 `phone`（首版不上架平板，规避适配驳回；后续做平板布局再加回）
- [ ] **重新生成调试签名**：包名变更后旧调试 Profile 已失效。DevEco Studio → File → Project Structure → Signing Configs → 勾选 Automatically generate signature，重新自动签名后才能真机调试。
- [ ] **AGC 创建应用**：我的项目 → 添加应用 → 平台选 APP(HarmonyOS)，名称填"码到成功"（重名则换"飞码传书""码上就到""青鸟传码"），包名填 `com.smsbridge.app`，应用分类选"应用"（分类创建后不可修改）。
- [ ] **生成发布密钥**：DevEco Studio → Build → Generate Key and CSR，新建 .p12 密钥库（密码至少 8 位、含两种以上字符类型）+ .csr 文件。**私钥务必备份，丢失后将无法给应用发新版本。**
- [ ] **申请发布证书**：AGC → 用户与访问 → 证书管理 → 新增证书 → 类型选"发布证书"，上传 .csr，下载 .cer。
- [ ] **申请发布 Profile**：AGC → 我的项目 → 对应应用 → HAP Provision Profile 管理 → 添加，类型选"发布"，设备类型勾选与 module.json5 的 deviceTypes 一致，下载 .p7b。
- [ ] **配置发布签名**：Project Structure → Signing Configs，取消 Automatically generate signature，手动填入 .p12（Store File/密码/别名）+ .cer（Certpath）+ .p7b（Profile），签名算法 SHA256withECDSA。
- [ ] **构建 Release 包**：Build → Build Hap(s)/APP(s) → Build APP(s)（默认即 Release 模式）。
- [ ] **验证产物**：`entry/build/default/outputs/default/*.app`；用 Build → Analyze HAP/APP 确认包名、版本号 1.0.0(1000000)、签名证书为发布证书。

## 三、AGC 应用信息与提审材料

- [x] 应用图标：`docs/release/appgallery-icon-1024.png`（AGC 上传用）与 `appgallery-icon-216.png`，1024/216、PNG 直角、无水印、<2MB；同步替换包内 `app_icon` / `icon` / `startIcon` 三处（512×512），与 AGC 上传图标一致（2026-09-01 完成）
- [ ] 截图 ≥3 张（建议 5 张，清单见 `docs/release/appgallery-listing.md`）
- [ ] 应用简介 + 详细描述（见 `docs/release/appgallery-listing.md`）
- [x] 隐私政策 URL：`https://nanhapy.github.io/sms-bridge/release/privacy-policy.html`（本仓库 GitHub Pages，开发者联系方式已填写：Larry Zhao / pzl1988p@163.com）
- [ ] 内容分级问卷
- [x] PC 接收端下载地址（填"了解更多"官网字段）：`https://github.com/nanhapy/sms-bridge/releases`（v2.0.3 安装包已上传，名称与新图标均已同步为"码到成功"骏马图标）
- [ ] 备案号 + 主办单位信息（版本信息 → 备案信息，点"校验证件号"通过）
- [ ] 版权证书上传（版本信息 → 版权信息）
- [ ] 上传 .app 软件包（版本信息 → 软件包，或 DevEco Studio → Build → Upload Product）
- [ ] 提交审核

## 四、常见驳回原因对照（提审前自查）

| 风险点 | 自查结论 |
|---|---|
| 隐私政策缺失/不合规 | URL 公网可访问，含数据类型、用途、存储、用户权利、联系方式 |
| 包名不一致 | AGC 包名 = bundleName = 备案包名 = Profile 绑定包名，四处一致 |
| 使用了调试签名 | 确认 .app 由发布证书签名（Analyze 验证） |
| 功能不完整 | 描述中说明 PC 端配套及下载地址；手机端界面在无电脑时有明确引导 |
| 权限申请过多 | 现仅 INTERNET / GET_NETWORK_INFO / READ_PASTEBOARD 三个普通权限，符合最小化 |
| 名称侵权/重名 | 创建应用时若提示占用则换备选名称 |

## 五、发布后

- [ ] .p12 私钥多副本备份（网盘 + U 盘）——丢失则该应用永久无法更新
- [ ] 关注审核反馈（常规约 24 小时，复杂应用 1–7 天），被驳回按意见修改后重新提交
- [ ] 版本更新流程：versionCode +1 → AGC 新建版本 → 上传新 .app → 重新提审

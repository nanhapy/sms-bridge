/**
 * 分享详情页（二级面板）：分享扩展被拉起时短暂展示的页面
 * 静态展示「正在发送验证码到电脑」，实际推送在 ShareExtAbility 中完成，
 * 处理完成后由扩展自动关闭本面板。
 */
@Entry
@Component
struct SharePanel {
  @State showIcon: boolean = true;

  build() {
    Column({ space: 16 }) {
      Text('📱')
        .fontSize(48)
      Text('正在发送验证码到电脑…')
        .fontSize(17)
        .fontWeight(FontWeight.Medium)
      Text('电脑端会弹出通知并自动复制到剪贴板')
        .fontSize(13)
        .fontColor('#909399')
      LoadingProgress()
        .width(36)
        .height(36)
        .color('#4D7CFE')
    }
    .width('100%')
    .height('100%')
    .justifyContent(FlexAlign.Center)
    .padding(24)
    .backgroundColor('#FFFFFF')
  }
}

# =============================================================
# 生成「青鸟传码」App 图标（渐变底 + 白色短信气泡）
# 输出：app_icon.png / icon.png / startIcon.png（均为 512x512）
# 依赖：.NET System.Drawing（Windows 自带）
# =============================================================
Add-Type -AssemblyName System.Drawing

function New-AppIcon([string]$path) {
  $S = 512
  $bmp = New-Object System.Drawing.Bitmap($S, $S)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = 'AntiAlias'
  $g.PixelOffsetMode = 'HighQuality'

  # 1. 圆角矩形渐变背景（蓝 -> 紫，对角线）
  $radius = 110
  $gp = New-Object System.Drawing.Drawing2D.GraphicsPath
  $d = $radius * 2
  $gp.AddArc(0, 0, $d, $d, 180, 90)
  $gp.AddArc($S - $d, 0, $d, $d, 270, 90)
  $gp.AddArc($S - $d, $S - $d, $d, $d, 0, 90)
  $gp.AddArc(0, $S - $d, $d, $d, 90, 90)
  $gp.CloseFigure()
  $c1 = [System.Drawing.Color]::FromArgb(61, 123, 255)
  $c2 = [System.Drawing.Color]::FromArgb(138, 92, 246)
  $brush = New-Object System.Drawing.Drawing2D.LinearGradientBrush(
    (New-Object System.Drawing.Point(0, 0)),
    (New-Object System.Drawing.Point($S, $S)),
    $c1, $c2)
  $g.FillPath($brush, $gp)

  # 2. 白色短信气泡（圆角矩形 + 左下尾巴 + 三个圆点）
  $white = [System.Drawing.Color]::White
  $wb = New-Object System.Drawing.SolidBrush($white)

  $bubble = New-Object System.Drawing.Drawing2D.GraphicsPath
  $bx = 110; $by = 140; $bw = 292; $bh = 210; $br = 48
  $bd = $br * 2
  $bubble.AddArc($bx, $by, $bd, $bd, 180, 90)
  $bubble.AddArc($bx + $bw - $bd, $by, $bd, $bd, 270, 90)
  $bubble.AddArc($bx + $bw - $bd, $by + $bh - $bd, $bd, $bd, 0, 90)
  $bubble.AddArc($bx, $by + $bh - $bd, $bd, $bd, 90, 90)
  $bubble.CloseFigure()
  $g.FillPath($wb, $bubble)

  # 尾巴（气泡左下角伸出的三角）
  $t1 = New-Object System.Drawing.Point(150, 336)
  $t2 = New-Object System.Drawing.Point(150, 404)
  $t3 = New-Object System.Drawing.Point(238, 336)
  $g.FillPolygon($wb, [System.Drawing.Point[]]@($t1, $t2, $t3))

  # 三个圆点（紫蓝色，与背景渐变呼应）
  $dotColor = [System.Drawing.Color]::FromArgb(94, 82, 255)
  $db = New-Object System.Drawing.SolidBrush($dotColor)
  $g.FillEllipse($db, 166, 225, 40, 40)
  $g.FillEllipse($db, 236, 225, 40, 40)
  $g.FillEllipse($db, 306, 225, 40, 40)

  $db.Dispose(); $bubble.Dispose(); $wb.Dispose(); $brush.Dispose(); $gp.Dispose(); $g.Dispose()
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Write-Host "OK $path"
}

$base = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
New-AppIcon (Join-Path $base 'AppScope\resources\base\media\app_icon.png')
New-AppIcon (Join-Path $base 'entry\src\main\resources\base\media\icon.png')
New-AppIcon (Join-Path $base 'entry\src\main\resources\base\media\startIcon.png')

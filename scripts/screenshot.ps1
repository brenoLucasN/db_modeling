# Abre o app e captura SÓ a janela dele, sem foco e sem mouse/teclado reais:
# - captura via PrintWindow (funciona com a janela atrás de outras);
# - interação via PostMessage para a própria janela (nunca vai para outra janela).
# Uso: screenshot.ps1 <exe> <png> <light|dark> [edit] [texto] [x,y]
param($Exe, $Out, $Mode = "dark", $Edit = "", $Keys = "", $Click = "", $Arg = "", $Pane = "")
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;using System.Runtime.InteropServices;
public class W{
[DllImport("user32.dll")]public static extern bool SetProcessDPIAware();
[DllImport("user32.dll")]public static extern bool GetClientRect(IntPtr h,out R r);
[DllImport("user32.dll")]public static extern bool PrintWindow(IntPtr h,IntPtr dc,uint f);
[DllImport("user32.dll")]public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
public struct R{public int L,T,Rt,B;}}
"@
[void][W]::SetProcessDPIAware()
Get-Process db-modeling -ErrorAction SilentlyContinue | Stop-Process -Force
$env:DBM_MODE = $Mode
if ($env:DBM_SELECT_ARG) { $env:DBM_SELECT = $env:DBM_SELECT_ARG } else { Remove-Item Env:DBM_SELECT -ErrorAction SilentlyContinue }
foreach ($k in "DBM_TRANSLUCENT") { $v = [Environment]::GetEnvironmentVariable("$($k)_ARG"); if ($v) { [Environment]::SetEnvironmentVariable($k, $v) } else { [Environment]::SetEnvironmentVariable($k, $null) } }
if ($env:DBM_SIZE_ARG) { $env:DBM_SIZE = $env:DBM_SIZE_ARG } else { Remove-Item Env:DBM_SIZE -ErrorAction SilentlyContinue }
if ($Edit -ne "") { $env:DBM_EDITOR = "1" } else { Remove-Item Env:DBM_EDITOR -ErrorAction SilentlyContinue }
if ($Pane -ne "") { $env:DBM_PANE = $Pane } else { Remove-Item Env:DBM_PANE -ErrorAction SilentlyContinue }
if ($Arg -ne "") { $p = Start-Process $Exe -ArgumentList "`"$Arg`"" -PassThru } else { $p = Start-Process $Exe -PassThru }
Start-Sleep 5
$p.Refresh(); $h = $p.MainWindowHandle
if ($h -eq [IntPtr]::Zero) { "ABORTADO: janela do app não encontrada"; exit 3 }
function LParam($x, $y) { [IntPtr](($y -shl 16) -bor ($x -band 0xFFFF)) }
function Click($x, $y) {
  [void][W]::PostMessage($h, 0x200, [IntPtr]0, (LParam $x $y))
  [void][W]::PostMessage($h, 0x201, [IntPtr]1, (LParam $x $y))
  [void][W]::PostMessage($h, 0x202, [IntPtr]0, (LParam $x $y))
  Start-Sleep -Milliseconds 500
}
function Type($text) {
  foreach ($ch in $text.ToCharArray()) {
    if ($ch -eq "`n") {
      [void][W]::PostMessage($h, 0x100, [IntPtr]0x0D, [IntPtr]1)   # WM_KEYDOWN Enter
      [void][W]::PostMessage($h, 0x101, [IntPtr]0x0D, [IntPtr]1)
    } else {
      [void][W]::PostMessage($h, 0x102, [IntPtr][int]$ch, [IntPtr]1) # WM_CHAR
    }
    Start-Sleep -Milliseconds 25
  }
  Start-Sleep -Milliseconds 800
}
if ($Edit -eq "type") {
  Click 480 400
  [void][W]::PostMessage($h, 0x100, [IntPtr]0x23, [IntPtr]1)  # End
  Type "`nTable tags {`nid int [pk]`nname text`n}`nRef: tags.id > users.id`n"
}
if ($Click -ne "") { $xy = $Click.Split(","); Click ([int]$xy[0]) ([int]$xy[1]) }
if ($Keys -ne "") { Type $Keys }
$r = New-Object W+R; [void][W]::GetClientRect($h, [ref]$r)
$bmp = New-Object Drawing.Bitmap ($r.Rt - $r.L), ($r.B - $r.T)
$g = [Drawing.Graphics]::FromImage($bmp); $dc = $g.GetHdc()
[void][W]::PrintWindow($h, $dc, 3)   # PW_CLIENTONLY | PW_RENDERFULLCONTENT
$g.ReleaseHdc($dc); $bmp.Save($Out)
Stop-Process -Id $p.Id -Force
"$($bmp.Width) x $($bmp.Height)"

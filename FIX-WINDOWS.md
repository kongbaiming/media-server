# 解决 Windows 智能应用控制阻止问题

## 方法1: 关闭智能应用控制

1. 打开 **Windows 安全中心**
2. 点击 **应用和浏览器控制**
3. 点击 **智能应用控制设置**
4. 选择 **关闭**

## 方法2: 添加排除项

1. 打开 **Windows 安全中心**
2. 点击 **病毒和威胁防护**
3. 点击 **管理设置**（在"病毒和威胁防护设置"下）
4. 向下滚动到 **排除项**
5. 点击 **添加或删除排除项**
6. 点击 **添加排除项** → **文件夹**
7. 选择项目文件夹：`D:\project\rust\media-server`
8. 也添加 cargo 目录：`C:\Users\你的用户名\.cargo`

## 方法3: 启用开发者模式

1. 打开 **设置** → **系统** → **开发者选项**
2. 打开 **开发人员模式**
3. 重启电脑

## 方法4: 使用 PowerShell 运行

以管理员身份打开 PowerShell，运行：

```powershell
# 临时禁用 SmartScreen
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process

# 然后运行
cd D:\project\rust\media-server
cargo run
```

## 方法5: 签名可执行文件（高级）

```powershell
# 创建自签名证书
$cert = New-SelfSignCertificate -Type CodeSigningCert -Subject "CN=MediaVault" -CertStoreLocation Cert:\CurrentUser\My

# 签名可执行文件
Set-AuthenticodeSignature -FilePath .\target\debug\media-server.exe -Certificate $cert
```

## 验证

关闭智能应用控制后，重新运行：

```bash
cd d:/project/rust/media-server
cargo run
```

应该看到：
```
Starting MediaVault Server...
Server listening on 0.0.0.0:8080
```

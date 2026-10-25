# Сборка TitoV3D Calc: Windows (.exe/.msi) и Android (.apk)

Готовые проекты лежат в `D:\open code\build\`:

```
build/
├─ tito-win/          Windows, Tauri 2 + Rust
│  ├─ src/index.html  наш фронтенд (копия из titov3d-calc)
│  └─ src-tauri/      Rust-часть, конфиг, иконки
├─ tito-android/      Android, Capacitor 8
│  ├─ www/index.html  тот же фронтенд
│  └─ android/        Gradle-проект
├─ keystore/          ключ подписи APK
└─ icon.png           исходник иконки 1024×1024
```

---

## 1. Требования (проверено на этой машине)

| Компонент | Версия |
|---|---|
| Windows | 11 Pro 26100 |
| Rust / Cargo | 1.99.1 (x86_64-pc-windows-msvc) |
| MSVC | Visual Studio Build Tools 2022 |
| WebView2 | 155.0.4283.62 |
| Node.js / npm | 24.22.1 / 11.19.1 |
| JDK (Gradle) | 21.0.8 (JBR из Android Studio) |
| Android SDK | platforms 33/34/35/36, build-tools 36.1, platform-tools 37 |
| Capacitor | 8.5.3 |
| Tauri CLI | 2.x |

Переменные окружения (уже прописаны в профиль пользователя):
`JAVA_HOME` → JDK 17 для сборки Tauri не нужен, для Android используется JBR 21;
`ANDROID_HOME` и `ANDROID_SDK_ROOT` → `%LOCALAPPDATA%\Android\Sdk`.

> В PowerShell `npm` может блокироваться политикой выполнения — вызывайте `npm.cmd`.

---

## 2. Windows: Tauri 2

### Сборка
```powershell
cd "D:\open code\build\tito-win"
npm install                 # один раз
npx tauri build
```

### Результат
```
src-tauri\target\release\tito-v3d-calc.exe              портативный exe
src-tauri\target\release\bundle\msi\TitoV3D Calc_6.1.0_x64_en-US.msi
src-tauri\target\release\bundle\nsis\TitoV3D Calc_6.1.0_x64-setup.exe
```

### Что внутри
- Окно 1120×840, минимум 900×640, центр экрана.
- `identifier` = `com.tito.v3dcalc`, версия 6.1.0.
- Иконка сгенерирована из `build/icon.png` командой `npx tauri icon`.
- `withGlobalTauri: true` — приложение определяет платформу и включает
  нативные ветки (экспорт файла).
- WebView2: если рантайма нет, установщик его подтянет
  (`webviewInstallMode: downloadBootstrapper`).

### Пересборка после правок фронтенда
```powershell
copy "D:\open code\titov3d-calc\index.html" "src\index.html"
npx tauri build
```

### Подпись
Без сертификата SmartScreen покажет «Неизвестный издатель». Для распространения
нужен code-signing сертификат; для личного использования не требуется.

---

## 3. Android: Capacitor 8

### Сборка debug (для проверки на телефоне)
```powershell
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
cd "D:\open code\build\tito-android"
npx cap sync android
cd android
.\gradlew.bat assembleDebug
```
Результат: `android\app\build\outputs\apk\debug\app-debug.apk`

### Сборка release с подписью
```powershell
$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
cd "D:\open code\build\tito-android\android"
.\gradlew.bat assembleRelease
```
Неподписанный APK: `app\build\outputs\apk\release\app-release-unsigned.apk`

Подпись:
```powershell
$bt = "$env:LOCALAPPDATA\Android\Sdk\build-tools\36.1.0"
& "$bt\zipalign.exe" -p -f 4 app-release-unsigned.apk app-release-aligned.apk
& "$bt\apksigner.bat" sign `
  --ks "D:\open code\build\keystore\tito-release.jks" `
  --ks-key-alias tito --ks-pass pass:TitoV3D_2026 --key-pass pass:TitoV3D_2026 `
  --out app-release.apk app-release-aligned.apk
```

### Ключ
```
build\keystore\tito-release.jks   alias: tito
```
Пароль — в `build\keystore\keystore-password.txt`.
**Сохраните ключ**: без него нельзя выпустить обновление поверх установленной версии.

### Параметры приложения
- `applicationId` = `com.tito.v3dcalc`
- `versionCode` = 610, `versionName` = "6.1.0"
- `minSdk` 24 (Android 7.0), `targetSdk` 36
- Экран: адаптивный (портрет и ландшафт), safe-area учтён
- Иконки: mipmap-* скопированы из набора Tauri

### Установка на телефон
1. Скинуть `app-release.apk` на телефон (Telegram, кабель, Drive).
2. Разрешить установку из неизвестных источников.
3. Установить. Данные хранятся внутри приложения (WebView localStorage).

### Google Play
- загружать `.aab`: `.\gradlew.bat bundleRelease`
- нужен targetSdk 35+ (у нас 36), политика конфиденциальности и форма Data Safety:
  приложение не собирает и не передаёт данные
- подпись — тот же keystore (лучше включить Play App Signing)

---

## 4. Чем нативная версия отличается от браузерной

| Возможность | Браузер | Windows (.exe) | Android (.apk) |
|---|---|---|---|
| Расчёты, история, каталог | ✅ | ✅ | ✅ |
| Хранение данных | localStorage | localStorage в WebView2 | localStorage в WebView |
| Экспорт JSON/CSV | скачивание файла | скачивание файла | сохранение через «Поделиться» |
| Импорт JSON | ✅ | ✅ | ✅ (системный выбор файла) |
| Резервная копия внутри | ✅ | ✅ | ✅ |
| Печать / PDF | ✅ | ⚠️ через диалог печати WebView2 | ❌ нет (используйте экспорт) |
| Работа офлайн | ✅ | ✅ | ✅ |

Если на Android плагин «Поделиться» недоступен, приложение откроет окно с текстом
файла и кнопкой «Скопировать» — данные не потеряются.

---

## 5. Если что-то не собирается

| Симптом | Решение |
|---|---|
| `npm` не запускается в PowerShell | вызывать `npm.cmd` или `Set-ExecutionPolicy -Scope Process Bypass` |
| `cargo: command not found` | `$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"` |
| Ошибка линковки Rust | доустановить MSVC: `Visual Studio Installer → C++ Build Tools` |
| Gradle не видит SDK | `android\local.properties` → `sdk.dir=C:/Users/<вы>/AppData/Local/Android/Sdk` |
| Gradle: `Unsupported class file major version` | `JAVA_HOME` должен указывать на JDK 21 (JBR Android Studio) |
| `SDK location not found` | проверить `ANDROID_HOME` и `local.properties` |
| Сборка Tauri падает на иконке | `npx tauri icon build\icon.png` заново |
| APK не ставится | удалить старую версию (другая подпись) или поднять `versionCode` |

---

## 6. Обновление версии

1. Поменять `APP_VERSION` в `index.html` и `<div class="v">`.
2. Скопировать `index.html` в `tito-win\src\` и `tito-android\www\`.
3. `npx tauri build` (Windows) и `npx cap sync android && gradlew assembleRelease` (Android).
4. Поднять `versionCode` (600 → 601) и `versionName`.

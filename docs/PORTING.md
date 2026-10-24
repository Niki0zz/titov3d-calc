# План переноса TitoV3D Calc на Windows (.exe) и Android (.apk)

Текущее состояние: однофайловое веб-приложение `index.html` (86 КБ), без зависимостей,
данные в `localStorage`, экспорт/импорт JSON, печать через `window.print()`.
Переписывать интерфейс и логику не нужно — нужна «обёртка» и адаптация трёх мест:
хранение файлов, скачивание файлов, печать.

---

## 1. Выбор технологии

| Вариант | Windows | Android | Размер | Трудозатраты |
|---|---|---|---|---|
| **Tauri 2 + Capacitor 7** (рекомендую) | ✅ .exe/.msi | ✅ .apk/.aab | 5–10 МБ | ~25–35 ч |
| Electron + Capacitor | ✅ .exe | ✅ .apk | 90–150 МБ | ~25 ч |
| Tauri 2 + Tauri Mobile | ✅ | ⚠️ мобильная часть сырая | 5–10 МБ | выше риск |
| PWA (без магазинов) | ярлык | «Установить» из Chrome | 0 | 1–2 ч |
| Flutter / Kotlin-переписывание | ✅ | ✅ | 15 МБ | 100+ ч, всё заново |

Почему Tauri, а не Electron: тот же WebView2, который уже есть в Windows 10/11,
итоговый установщик ~3 МБ вместо ~100 МБ, памяти ест в 3–4 раза меньше.
Почему Capacitor, а не Cordova: живая поддержка, Gradle 8 / SDK 35, нормальные плагины.

Один и тот же `index.html` кладётся в оба проекта — расхождений в логике не будет.

---

## 2. Подготовка кода (обязательный этап, 3–4 ч)

1. **Разбить файл** на `index.html`, `styles.css`, `app.js`. Не обязательно для работы,
   но упрощает сборку, кэш и отладку в Android Studio.
2. **Адаптер хранения.** Сейчас `localStorage`. В приложениях нужно:
   - писать `tito_v3d_calc.json` в каталог данных приложения (Tauri: `appDataDir`,
     Capacitor: `Directory.Data`);
   - держать копию в `localStorage` для мгновенного старта;
   - при расхождении побеждает файл с более новой `saved_at`.
3. **Экспорт файлов.** `Blob` + `<a download>` в Android WebView **не работает**.
   Нужен плагин: `Filesystem.writeFile` + `Share.share`. На Windows Tauri —
   `dialog.save` + `fs.writeTextFile`.
4. **Печать.** `window.print()` в WebView2 работает нестабильно, в Android WebView
   не работает вовсе. Варианты:
   - Android: `PrintManager` через плагин (`@capawesome/capacitor-printer`) или
     «Сохранить PDF» через `Filesystem` + `Share`;
   - Windows: печать через WebView2 `ShowPrintUI`, либо тот же PDF-экспорт.
   Резервный путь одинаковый для обеих платформ — генерировать PDF из HTML.
5. **Кнопка «Назад»** на Android: `@capacitor/app` → `backButton` → переключение вкладок,
   выход только с вкладки «Калькулятор».
6. **Мелочи:** `viewport-fit=cover` уже есть, safe-area учтён; добавить статус-бар,
   блокировку поворота (портрет) или адаптив (уже адаптивен), splash + иконки.

---

## 3. Windows: Tauri 2

### Требования
- Node.js 20+, Rust (rustup), Visual Studio Build Tools с workload «C++ Build Tools»
- WebView2 Runtime (в Windows 10/11 предустановлен)

### Команды
```bash
npm create tauri-app@latest titov3d-win -- --template vanilla
cd titov3d-win
npm i
# заменить src/ на наши файлы: index.html, styles.css, app.js
npx tauri icon path/to/icon.png        # генерирует все размеры, включая .ico
npx tauri build
```

### Конфиг `src-tauri/tauri.conf.json`
```json
{
  "productName": "TitoV3D Calc",
  "identifier": "com.tito.v3dcalc",
  "version": "5.1.0",
  "build": { "frontendDist": "../src" },
  "app": {
    "windows": [{
      "title": "TitoV3D Calc",
      "width": 1120, "height": 820,
      "minWidth": 900, "minHeight": 640,
      "resizable": true
    }]
  },
  "bundle": {
    "targets": ["msi", "nsis"],
    "icon": ["icons/icon.ico"],
    "windows": { "webviewInstallMode": { "type": "downloadBootstrapper" } }
  }
}
```

### Результат
- `src-tauri/target/release/bundle/msi/TitoV3D Calc_5.1.0_x64_en-US.msi`
- `.../nsis/TitoV3D Calc_5.1.0_x64-setup.exe`
- `.../release/titov3d-calc.exe` — портативная версия

### Плагины
```bash
npm i @tauri-apps/plugin-fs @tauri-apps/plugin-dialog
```
Права в `capabilities/default.json`: `fs:allow-appdata-read-recursive`,
`fs:allow-appdata-write-recursive`, `dialog:allow-save`, `dialog:allow-open`.

### Подпись
Без сертификата SmartScreen покажет «Неизвестный издатель». Сертификат кода
(EV/OV) — от ~15 000 ₽/год. Для внутреннего использования можно не подписывать.

---

## 4. Android: Capacitor 7

### Требования
- JDK 17, Android Studio (SDK Platform 35/36, Build-Tools, Platform-Tools)
- ~4 ГБ свободного места

### Команды
```bash
npm init -y
npm i @capacitor/core @capacitor/cli
npm i @capacitor/android @capacitor/filesystem @capacitor/share @capacitor/app
npm i @capacitor/status-bar @capacitor/keyboard
npx cap init "TitoV3D Calc" com.tito.v3dcalc --web-dir=www
mkdir www && copy ..\index.html www\index.html
npx cap add android
npx cap sync
npx cap open android        # дальше сборка в Android Studio
```

### `capacitor.config.json`
```json
{
  "appId": "com.tito.v3dcalc",
  "appName": "TitoV3D Calc",
  "webDir": "www",
  "android": {
    "allowMixedContent": false,
    "captureInput": true,
    "webContentsDebuggingEnabled": false
  },
  "plugins": {
    "SplashScreen": { "launchAutoHide": true, "backgroundColor": "#0b0d10" }
  }
}
```

### `android/app/build.gradle`
```gradle
android {
  defaultConfig {
    applicationId "com.tito.v3dcalc"
    minSdk 23
    targetSdk 35
    versionCode 51
    versionName "5.1.0"
  }
  signingConfigs {
    release {
      storeFile file("tito-release.keystore")
      storePassword System.getenv("TITO_STORE_PASS")
      keyAlias "tito"
      keyPassword System.getenv("TITO_KEY_PASS")
    }
  }
  buildTypes {
    release { signingConfig signingConfigs.release; minifyEnabled false }
  }
}
```

### Ключ и сборка
```bash
keytool -genkeypair -v -keystore tito-release.keystore -alias tito \
        -keyalg RSA -keysize 2048 -validity 10000
cd android
./gradlew assembleDebug      # app/build/outputs/apk/debug/app-debug.apk
./gradlew assembleRelease    # подписанный APK
./gradlew bundleRelease      # AAB для Google Play
```

### Результат
- `android/app/build/outputs/apk/release/app-release.apk` (~8–12 МБ)
- `android/app/build/outputs/bundle/release/app-release.aab`

---

## 5. Подводные камни Android (проверено по опыту таких портов)

| Проблема | Что делать |
|---|---|
| `<a download>` + Blob не скачивает | только `Filesystem` + `Share` |
| `window.print()` ничего не делает | `PrintManager`-плагин или PDF через `Share` |
| `localStorage` может быть вычищен системой | дублировать в файл `Directory.Data` |
| Кнопка «Назад» закрывает приложение | `App.addListener('backButton')` |
| `alert/confirm` выглядят чужеродно | заменить на свои модалки (у нас 12 вызовов `confirm`) |
| Ввод числа на некоторых клавиатурах | `inputmode="decimal"` уже стоит |
| File input для импорта | работает, но надёжнее `@capawesome/capacitor-file-picker` |
| Экран блокировки/сворачивание | данные уже сохранены, ничего не теряется |

---

## 6. Общая схема хранения (одинаково на обеих платформах)

```
[UI] → saveData() → localStorage (мгновенно)
                 ↘ debounce 500 мс → файл tito_v3d_calc.json
[старт] → читаем файл → сравниваем saved_at → более новый побеждает
[экспорт] → диалог сохранения → 3d_calc_data_v2.json (совместим с Python-версией)
```

Миграции `meta.catalog` / `meta.printers` уже реализованы и переносятся без изменений.
Формат JSON остаётся совместимым с десктопной Tkinter-версией.

---

## 7. Этапы и оценка

| # | Этап | Часы |
|---|---|---|
| 0 | Разбить на файлы, адаптер хранения, замена download/print | 3–4 |
| 1 | Tauri: скаффолд, конфиг, иконки, сборка MSI/EXE | 4–6 |
| 2 | Capacitor: скаффолд, Gradle, сборка APK | 5–7 |
| 3 | Плагины: файлы, share, печать, back, статус-бар | 4–6 |
| 4 | Свои модалки вместо confirm, полировка | 3–4 |
| 5 | Тесты на реальных устройствах (Win 10/11, Android 9/12/15) | 3–4 |
| 6 | Подпись, версии, релиз, инструкция | 2–3 |
| | **Итого** | **24–34 ч** |

---

## 8. Риски

1. **Печать** — единственное место, где нужен нативный код. Если печать не критична,
   можно оставить только PDF-экспорт и сэкономить 3–4 ч.
2. **Google Play** — ревью 2–7 дней, нужен target SDK 35+, AAB, политика
   конфиденциальности и форма Data Safety (у нас данных не собираем — заполняется
   за 5 минут). Для внутреннего использования проще раздавать APK напрямую.
3. **SmartScreen** на неподписанном `.exe` — предупреждение при первом запуске.
4. **Размер APK** — 8–12 МБ из-за WebView-обвязки; это нормально.
5. **Обновления** — Tauri updater (автообновление) либо новая версия APK вручную.

---

## 9. Что делаем дальше

Могу выполнить по шагам:

1. **Подготовка** (без установки инструментов): разбить на `index.html` + `styles.css` +
   `app.js`, добавить слой хранения с фолбэком — приложение остаётся работоспособным
   как обычный файл.
2. **Windows-скаффолд**: готовый проект Tauri с конфигом, иконками и инструкцией
   `build.bat`.
3. **Android-скаффолд**: проект Capacitor с плагинами, `build.gradle` и скриптом
   сборки APK.
4. **Сборка здесь**: возможна только после установки Rust (~1.5 ГБ) и Android SDK
   (~4 ГБ) — скажите, и я разверну.

Быстрый компромисс на сегодня: PWA-версия (GitHub Pages / локальный сервер) —
устанавливается на Android как приложение и открывается ярлыком в Windows без сборки.

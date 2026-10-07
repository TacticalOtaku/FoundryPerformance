# Сплэш обновления и иконка «Клавиша» (дизайн)

Дата: 2026-10-07 · Статус: утверждён в обсуждении, ждёт вычитки

Подпроект 4 из четырёх («1 → 3 → 2 → 4»). После «Намерения» (`2026-10-07-intent-redesign-design.md`)
от подпроекта осталось окно обновления. Решено делать его сплэшем при запуске, как у Discord.
Заодно иконка приложения переходит с ретро-ручки пульта на стиль Tactile.

Макеты, по которым принимали решения: `docs/superpowers/mockups/2026-10-07-icon-concepts.html`
(выбран вариант 1 «Клавиша») и `docs/superpowers/mockups/2026-10-07-splash-states.html`
(состояния сплэша, вторая версия со свечением без обрезки).

## 1. Решения пользователя

- Форма — **сплэш при запуске**, а не отдельное окно по клику и не лист внутри лаунчера.
- Найденное обновление **ставится само**, без вопроса. «Что нового» потом показывает фишка версии.
- Если лаунчер уже открыт: фоновая проверка и пилюля «Обновить до X» остаются. Клик по пилюле
  закрывает лаунчер и открывает **тот же сплэш**. Путь установки один.
- Подход **A**: отдельное окно `splash`, логика в Svelte. Существующие `check_update` и
  `apply_update` используются повторно, проверки в `update.rs` не меняются.
- Таймаут проверки на сплэше ≈ 4 с. Сплэш стоит и перед запуском игры по адресу из аргументов.
- Иконка — концепция 1 «Клавиша». Кольцо иконки на сплэше служит индикатором прогресса.
- Свечение кольца повторяет его форму и не обрезается границей SVG.
- Сквозная проверка обновления через локальную ленту в debug-сборке — в объёме.
- Работа идёт в ветке `feat/splash` от `main` (`54389d7`).

## 2. Поток

```
запуск (режим Launcher)
  └─ окно splash ── check_update(4 с)
        ├─ нет обновления ───────────────► splash_done → лаунчер | игра по адресу
        ├─ нет связи ── 1,5 с ───────────► splash_done
        ├─ найдено ── apply_update
        │     ├─ загрузка 0…99 %  («Пропустить» → отмена → splash_done)
        │     ├─ 100 % → проверка хеша, подписи, версии
        │     ├─ ошибка ──► экран ошибки, ждёт «Открыть лаунчер» → splash_done
        │     └─ замена exe → перезапуск с исходными аргументами + --updated
        └─ (запуск с --updated) ── 0,6 с «Обновлено до X» ─► splash_done

лаунчер открыт, пилюля «Обновить до X»
  └─ restart_to_update → лаунчер закрыт, открыт splash, действие = «лаунчер»
```

Установщик (`Mode::Install`) и удаление (`Mode::Uninstall`) идут мимо сплэша.

## 3. Rust

### 3.1 Окно

`windows::open_splash(app)`: метка `splash`, `WebviewUrl::App("index.html")`, 300 × 340 логических
пикселей, `decorations(false)`, `resizable(false)`, `center()`, в панели задач (экран ошибки ждёт
клика и не должен теряться под другими окнами),
`visible(false)` → `show()` после построения. Положение окна не запоминается (`WindowMemory`
роль ему не выдаёт). Константа `SPLASH = "splash"` рядом с `LAUNCHER` и `GAME`.

### 3.2 Действие после сплэша

```rust
pub enum AfterSplash { Launcher, Adhoc { url: String, mode: LaunchMode } }
pub struct SplashState(Mutex<AfterSplash>);
```

В `setup` для `Mode::Launcher` вместо `open_launcher` / `launch_adhoc`:
`parse_cli` → `AfterSplash` → `app.manage(SplashState)` → `open_splash`. Трей создаётся как сейчас.

### 3.3 Команды

- `check_update(timeout_s: Option<u64>)`: таймаут ленты, по умолчанию 8 с. Сплэш передаёт 4.
  `fetch_manifest` получает таймаут параметром.
- `apply_update`: без изменений, кроме двух правок.
  - **Отмена.** `UpdateState` получает `cancelled: AtomicBool`. Сбрасывается в начале
    `apply_update`. `download` проверяет его между чанками, `apply_update` — перед `verify` и перед
    `apply`. Отменённая загрузка возвращает `update.err.cancelled`.
  - **Перезапуск.** Новый exe запускается с исходными аргументами процесса плюс `--updated`
    (без повторного `--updated`). Чистая функция `relaunch_args(original) -> Vec<String>`.
    Так портативная сборка с `--portable` без маркера не запустится установщиком, а запуск по
    адресу не потеряет адрес.
- `splash_done`: забирает `AfterSplash`, открывает лаунчер или вызывает `launch_adhoc`, затем
  `destroy()` окна `splash`. Ошибка `launch_adhoc` открывает лаунчер, чтобы не остаться без окон.
- `restart_to_update`: ставит `AfterSplash::Launcher`, открывает `splash`, затем `destroy()`
  лаунчера. Программный `destroy()` не порождает `CloseRequested`, приложение не выходит.
- `splash_flags` → `{ updated: Option<String> }`: версия текущего exe, если процесс запущен
  с `--updated`, иначе `None`. Значение отдаётся один раз (хранится в `SplashState` и забирается
  `take()`): сплэш, открытый пилюлей в том же процессе, не покажет «Обновлено» повторно.
- `cancel_update`: ставит флаг отмены. «Пропустить» вызывает `cancel_update`, затем `splash_done`.

### 3.4 Закрытие

`CloseRequested` окна `splash` (Alt+F4): если идёт замена exe (флаг `applying` в `UpdateState`,
выставлен от `verify` до перезапуска), закрытие запрещается через `api.prevent_close()`.
Иначе — выход, как у лаунчера.

### 3.5 Права

Окно `splash` добавляется в `windows` capability `launcher.json`. Новые права:
`allow-splash-done`, `allow-restart-to-update`, `allow-splash-flags`, `allow-cancel-update`.

### 3.6 Локальная лента (только debug)

В `cfg!(debug_assertions)` переменная окружения `FP_UPDATE_FEED` задаёт адрес ленты, а
разрешённый префикс загрузки становится каталогом этой ленты. При заданной переменной
`check_update` в debug не возвращает «нет обновлений». В release-сборке переменная
игнорируется: чистая функция `local_feed(dev, env)` вызывается с `dev = cfg!(debug_assertions)`
и проверяется тестом. Проверки хеша, подписи и версии не меняются.

## 4. Svelte

### 4.1 Вход

`App.svelte` проверяет метку окна (`getCurrentWindow().label === "splash"`; в превью — `?splash`).
Для сплэша рендерится только `SplashScreen` в собственном `.tc-root`: без `TitleBar`, пилюль и
подсказок. Тема, акцент и язык берутся из `get_state`, как в лаунчере. Перетаскивание — весь корень
`data-tauri-drag-region`.

### 4.2 `src/lib/splash.ts`

Чистая машина состояний, без DOM и без `api` (зависимости передаются параметрами, таймеры —
через переданный `schedule`).

| Фаза | Кольцо | Заголовок | Подстрока | Кнопка |
|---|---|---|---|---|
| `checking` | `spin` | «Проверяю обновления» | `v0.2.2` | «Пропустить» |
| `downloading(pct)` | `progress` | «Скачиваю {version}» | `{pct} %` | «Пропустить» |
| `verifying` (pct = 100) | `breathe` | «Проверяю подпись» | «затем перезапуск» | — |
| `restarting` | `breathe` | «Перезапускаю» | `v{version}` | — |
| `offline` | `muted` | «Нет связи с GitHub» | «открываю лаунчер…» | — |
| `error(key)` | `error` | текст `update.err.*` | «установка отменена, версия {current} цела» | «Открыть лаунчер» |
| `updated(version)` | `full` | «Обновлено до {version}» | — | — |

Переходы:

- `checking` + «нет обновления» → `splash_done` сразу;
- `checking` + ошибка проверки → `offline` → через 1,5 с `splash_done`;
- `checking` + найдено → `apply_update`, фаза `downloading(0)`;
- `update-progress` = 100 → `verifying`;
- без `Content-Length` событий прогресса нет: фаза остаётся `downloading` с кольцом `spin` до
  ответа `apply_update`;
- успешный `apply_update` → `restarting`; процесс завершает Rust;
- ошибка `apply_update`, кроме `update.err.cancelled` → `error(key)`;
- старт с `splash_flags.updated` → `updated(version)` → через 0,6 с `splash_done`.

### 4.3 `src/components/splash/Ring.svelte`

SVG 150 × 150: дорожка `--tc-sunken`, кольцо r 60 толщиной 12 в `--tc-accent`, поднятая клавиша
r 38 с тенью и персиковой точкой r 10. Режимы `spin` / `progress` / `breathe` / `muted` / `error` /
`full`.

- **Свечение** — SVG-фильтр `feGaussianBlur` с областью `x="-50%" y="-50%" width="200%"
  height="200%"`, у `svg` — `overflow: visible`. CSS `drop-shadow` не используется: он обрезается
  границей SVG квадратом.
- `progress`: `stroke-dasharray` от `pct`, старт сверху (`rotate(-90)`), переход 200 мс.
- `spin`: вращение дуги 90° за 1,1 с. При «Движение: меньше» дуга стоит и пульсирует прозрачностью.
- `error`: кольцо и точка `--tc-danger`. `muted`: пунктир `--tc-muted`.
- Анимируются только `transform`, `opacity` и `stroke-dashoffset`/`dasharray`.

### 4.4 `src/screens/SplashScreen.svelte`

Кольцо сверху (отступ 54 px), заголовок 15 px/600, подстрока моно 12,5 px `--tc-muted`, кнопка
внизу — ghost-пилюля. Заголовок в `role="status"` `aria-live="polite"`. Кольцо — `role="progressbar"`
с `aria-valuenow` в фазе `downloading`, иначе `aria-hidden`.

### 4.5 Лаунчер

- `UpdatePill` по клику вызывает `updateApi.restartToUpdate()`.
- Из стора удаляются `applyUpdate` и `updating`, из словарей — `update.progress`.
- `VersionChip` и фоновая проверка раз в 30 минут не меняются.

### 4.6 i18n

Новые ключи в RU и EN: `splash.checking`, `splash.downloading`, `splash.verifying`,
`splash.verifyingHint`, `splash.restarting`, `splash.offline`, `splash.openingLauncher`,
`splash.updated`, `splash.skip`, `splash.openLauncher`, `splash.keptVersion`,
`update.err.cancelled` (на экран не выводится).

### 4.7 Мок

`?splash` открывает сплэш в превью. `update`: `1` — найдено и успешно, `fail` — нет связи,
`bad` — ошибка подписи на 100 %, `updated` — старт после обновления. Без параметра — обновлений нет.

## 5. Иконка

- `app-icon.svg` переписывается на «Клавишу». Плитка 1024 с радиусом 22 % (как `rx 56` из 256),
  градиент `#343D58 → #1D2333`, кант `#fff` 8 %. Персиковое кольцо — градиент `#FFA275 → #E08256`
  (OKLCH 0,80 → 0,70 при c 0,13, h 45). Свечение `#F59569`. Клавиша — градиент `#3B4563 → #262D40`
  с тенью, точка в центре — тот же градиент персика.
- `npm run icon` (resvg → `app-icon.png` → `tauri icon`) пересобирает все размеры и `icon.ico`.
  Трей берёт `default_window_icon` и обновляется сам.
- После генерации 16 и 32 px смотрятся глазами. Если свечение и тень мажут, для ≤ 32 px
  делается упрощённый `app-icon-small.svg` (без свечения, кольцо толще), а `.ico` собирается из
  двух исходников.
- Иконка всегда персиковая и от акцента пользователя не зависит.
- Кэш иконок Windows может держать старую иконку на ярлыках. Упоминание идёт в `CHANGELOG.md`.

## 6. Проверка

- **Vitest:** все переходы `splash.ts` из §4.2, включая «Пропустить» в загрузке и её отсутствие
  после 100 %; паритет i18n; `usage.test.ts` с новыми ключами.
- **Rust:** `relaunch_args` (исходные + `--updated`, без дублей); отмена рвёт загрузку и не доходит
  до `apply`; `AfterSplash` из аргументов; таймаут передаётся; `FP_UPDATE_FEED` не действует вне
  debug.
- **Гейты:** `npm test`, `svelte-check`, `cargo test`, `cargo clippy -- -D warnings`, build.
- **Превью:** все фазы сплэша в обеих темах на персике и шалфее; «Движение: меньше»; пилюля при
  `?update=1`. Скриншоты пользователю.
- **Реальная сборка** с изолированными `APPDATA` и `LOCALAPPDATA`:
  - портативный старт: сплэш → «нет связи» или «нет обновлений» → лаунчер;
  - иконка в панели задач, трее и Проводнике;
  - GPU в простое после сплэша ≈ 0 %.
- **Сквозное обновление:** debug-сборка с `FP_UPDATE_FEED` на локальную ленту из `dist-release`
  (подписанный exe, `latest.json` от `make-manifest.mjs`), раздаваемую с `localhost`. Ожидается:
  загрузка → проверка → замена → перезапуск с `--updated` → «Обновлено до X» → лаунчер.

## 7. Вне объёма

- Отдельное окно «Что нового» после обновления (его роль играет подсказка `VersionChip`).
- Настройка «Спрашивать перед обновлением».
- Дельта-обновления и откат на `.old`.
- Релиз 0.2.3 — только с разрешения пользователя.
